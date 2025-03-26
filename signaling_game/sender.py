from typing import Literal

import torch
from torch import nn, Tensor

from compiler.ll_one.grammar import (
    Grammar as LL1Grammar,
    EPSILON_SYMBOL,
    NonTerminalSymbol,
    START_SYMBOL,
    Symbol,
    TerminalSymbol,
)


PAD_INDEX = 0
BOS_INDEX = 1
EOS_INDEX = 2
SPECIAL_TOKENS_NUM = 3


class Sender(nn.Module):
    def __init__(
        self,
        input_vocab_size: int,
        hidden_dim: int,
        message_vocab_size: int,
        message_max_seq_len: int,
        m_h_0_type: Literal["x_embedding", "none"],
        message_grammar: LL1Grammar | None,
        terminal2vocab_size: dict[TerminalSymbol, int] | None,
    ):
        super(Sender, self).__init__()

        self.input_vocab_size = input_vocab_size
        self.hidden_dim = hidden_dim
        self.message_vocab_size = message_vocab_size + SPECIAL_TOKENS_NUM
        self.message_max_seq_len = message_max_seq_len
        self.m_h_0_type = m_h_0_type
        self.message_grammar = message_grammar

        self.x_token_embedding = nn.Embedding(
            input_vocab_size, hidden_dim, padding_idx=PAD_INDEX
        )
        self.x_rnn = nn.LSTM(hidden_dim, hidden_dim, batch_first=True)

        if message_grammar is None:
            self.message_token_embedding = nn.Linear(
                self.message_vocab_size, hidden_dim
            )
            self.message_rnn = nn.LSTM(hidden_dim, hidden_dim, batch_first=True)
            self.message_output_layer = nn.Linear(hidden_dim, self.message_vocab_size)
        elif isinstance(message_grammar, LL1Grammar):
            assert terminal2vocab_size is not None
            assert sum(terminal2vocab_size.values()) == message_vocab_size
            self.terminal2vocab_size = terminal2vocab_size
            self.rule_id2rule: dict[int, tuple[NonTerminalSymbol, list[Symbol]]] = {}
            start = 0
            for lhs, rhs in message_grammar.rules.items():
                for rule in rhs:
                    if rule == EPSILON_SYMBOL:
                        raise ValueError("生成規則にεが含まれています")
                    self.rule_id2rule[start] = (lhs, rule)
                    start += 1
            self.symbol2id = {
                k: i
                for i, k in enumerate(
                    message_grammar.non_terminals | message_grammar.terminals
                )
            }
            self.PAD_RULE_INPUT_INDEX = len(self.symbol2id)
            self.PAD_RULE_OUTPUT_INDEX = self.message_vocab_size + len(
                self.rule_id2rule
            )

            self.id2symbol = {v: k for k, v in self.symbol2id.items()}
            self.terminal2id_span = {}
            start = SPECIAL_TOKENS_NUM
            for k, v in terminal2vocab_size.items():
                self.terminal2id_span[k] = (start, start + v)
                start += v

            self.grammar_mask = torch.ones(
                (
                    len(self.symbol2id) + 1,
                    self.message_vocab_size + len(self.rule_id2rule) + 1,
                ),
                device="cpu",
            ) * -float("inf")
            for terminal, (start, end) in self.terminal2id_span.items():
                self.grammar_mask[
                    self.symbol2id[terminal],
                    start:end,
                ] = 0
            for rule_id, (lhs, rule) in self.rule_id2rule.items():
                lhs_idx = self.symbol2id[lhs]
                if isinstance(rule, list):
                    self.grammar_mask[lhs_idx, rule_id + self.message_vocab_size] = 0
                else:
                    raise ValueError("生成規則にεが含まれています")
            self.grammar_mask[self.PAD_RULE_INPUT_INDEX, self.PAD_RULE_OUTPUT_INDEX] = 0

            self.symbol_embedding = nn.Embedding(
                len(self.symbol2id) + 1, hidden_dim, padding_idx=len(self.symbol2id)
            )
            self.message_input_embedding = nn.Linear(
                self.message_vocab_size + len(self.rule_id2rule) + 1, hidden_dim
            )
            input_dim = (
                hidden_dim * 2 if m_h_0_type == "x_embedding" else hidden_dim * 3
            )
            self.message_rnn = nn.LSTM(input_dim, hidden_dim, batch_first=True)
            self.message_output_layer = nn.Linear(
                hidden_dim, self.message_vocab_size + len(self.rule_id2rule) + 1
            )

    def forward(
        self, x: Tensor, x_pad_mask: Tensor
    ) -> tuple[Tensor, Tensor, Tensor, Tensor]:
        h_n, c_n = self._calc_x_language_model_state(x, x_pad_mask)

        if self.message_grammar is None:
            return self._generate_message_without_grammar(h_n, c_n, x.size(0), x.device)
        elif isinstance(self.message_grammar, LL1Grammar):
            return self._generate_message_with_ll1_grammar(
                h_n, c_n, x.size(0), x.device
            )
        else:
            raise ValueError("Invalid message grammar")

    def _calc_x_language_model_state(
        self, x: Tensor, x_pad_mask: Tensor
    ) -> tuple[Tensor, Tensor]:
        x_token_embeddings = self.x_token_embedding(x)
        batch_size = x.size(0)
        h_n = torch.zeros(
            (1, batch_size, self.hidden_dim), device=x_token_embeddings.device
        )
        c_n = torch.zeros(
            (1, batch_size, self.hidden_dim), device=x_token_embeddings.device
        )
        final_h_n = torch.zeros(
            (1, batch_size, self.hidden_dim), device=x_token_embeddings.device
        )
        final_c_n = torch.zeros(
            (1, batch_size, self.hidden_dim), device=x_token_embeddings.device
        )
        final_n_flags = [False] * batch_size
        for n in range(x.size(1)):
            x_token_embedding = x_token_embeddings[:, n, :]
            _, (h_n, c_n) = self.x_rnn(x_token_embedding.unsqueeze(1), (h_n, c_n))
            for b in range(batch_size):
                if x_pad_mask[b, n].item():
                    if not final_n_flags[b]:
                        final_h_n[0, b] = h_n[0, b]
                        final_c_n[0, b] = c_n[0, b]
                        final_n_flags[b] = True
        # paddingがない場合は最後の隠れ状態を使う
        for b in range(batch_size):
            if not x_pad_mask[b, -1].item():
                final_h_n[0, b] = h_n[0, b]
                final_c_n[0, b] = c_n[0, b]
                final_n_flags[b] = True
        assert all(final_n_flags)
        return final_h_n, final_c_n

    def _generate_message_without_grammar(
        self, h_n: Tensor, c_n: Tensor, batch_size: int, device: torch.device
    ) -> tuple[Tensor, Tensor, Tensor]:
        assert self.message_grammar is None

        bos_one_hot_token = torch.ones(
            (batch_size, self.message_vocab_size), dtype=torch.float, device=device
        )
        bos_one_hot_token[:, BOS_INDEX] = 1
        m_token_embedding = self.message_token_embedding(bos_one_hot_token)
        m_history = (
            torch.ones((batch_size, 1), dtype=torch.long, device=device) * BOS_INDEX
        )
        m_padding_mask = torch.zeros((batch_size, 1), dtype=torch.bool, device=device)
        one_hot_m_history = torch.zeros(
            (batch_size, 1, self.message_vocab_size), device=device
        )
        one_hot_m_history[:, 0, BOS_INDEX] = 1
        entropy = torch.zeros(batch_size, device=device)
        while True:
            _, (h_n, c_n) = self.message_rnn(m_token_embedding.unsqueeze(1), (h_n, c_n))
            output = self.message_output_layer(h_n.squeeze(0))
            m_token_embedding, m_history, m_padding_mask, one_hot_m_history, entropy = (
                self._generate_next_token_without_grammar(
                    output, m_history, m_padding_mask, one_hot_m_history, entropy
                )
            )

            if (
                torch.all(
                    (m_history[:, -1] == EOS_INDEX) | (m_history[:, -1] == PAD_INDEX)
                ).item()
                or m_history.size(1) >= self.message_max_seq_len
            ):
                break

        entropy = entropy / (m_padding_mask == False).float().sum(dim=1)

        assert torch.all(one_hot_m_history[:, :, PAD_INDEX] == 0).item()
        return m_history, m_padding_mask, one_hot_m_history, entropy

    def _generate_next_token_without_grammar(
        self,
        output: Tensor,
        m_history: Tensor,
        m_padding_mask: Tensor,
        one_hot_m_history: Tensor,
        entropy: Tensor,
    ) -> tuple[Tensor, Tensor, Tensor, Tensor, Tensor]:
        assert self.message_grammar is None

        entropy_item = (
            -1.0 * torch.softmax(output, dim=1) * torch.log_softmax(output, dim=1)
        ).sum(dim=1)

        output[:, BOS_INDEX] = -float("inf")
        output[:, PAD_INDEX] = -float("inf")

        gumbel_softmax = torch.nn.functional.gumbel_softmax(
            output, tau=1, hard=True, dim=1
        )
        next_mask = (m_history[:, -1] == EOS_INDEX) | (m_history[:, -1] == PAD_INDEX)
        pad_embedding = torch.zeros(
            (output.size(0), self.message_vocab_size),
            dtype=torch.float,
            device=output.device,
        )
        next_one_hot = torch.where(
            next_mask.unsqueeze(1).expand_as(gumbel_softmax),
            pad_embedding,
            gumbel_softmax,
        )
        m_token_embedding = self.message_token_embedding(next_one_hot)

        m_history = torch.cat(
            [m_history, next_one_hot.argmax(dim=1, keepdim=True)], dim=1
        )
        m_padding_mask = torch.cat(
            [
                m_padding_mask,
                next_mask.unsqueeze(1),
            ],
            dim=1,
        )
        one_hot_m_history = torch.cat(
            [one_hot_m_history, next_one_hot.unsqueeze(1)], dim=1
        )

        # print(next_mask, entropy, entropy + entropy_item)
        entropy = torch.where(
            next_mask,
            entropy,
            entropy + entropy_item,
        )

        return m_token_embedding, m_history, m_padding_mask, one_hot_m_history, entropy

    def _generate_message_with_ll1_grammar(
        self,
        x_h_n: Tensor,
        x_c_n: Tensor,
        batch_size: int,
        device: torch.device,
    ) -> tuple[Tensor, Tensor, Tensor, Tensor]:
        assert isinstance(self.message_grammar, LL1Grammar)

        m_input_embedding = torch.zeros((batch_size, self.hidden_dim), device=device)
        S_RULE_INPUT_INDEX = self.symbol2id[START_SYMBOL]
        stack = [[S_RULE_INPUT_INDEX] for _ in range(batch_size)]
        if self.m_h_0_type == "x_embedding":
            h_n = x_h_n
            c_n = x_c_n
        elif self.m_h_0_type == "none":
            h_n = torch.zeros((1, batch_size, self.hidden_dim), device=device)
            c_n = torch.zeros((1, batch_size, self.hidden_dim), device=device)

        m_history_list = [[] for _ in range(batch_size)]
        one_hot_m_history_list = [[] for _ in range(batch_size)]
        entropy = torch.zeros(batch_size, device=device)
        while True:
            current = torch.tensor(
                [stack[b].pop() for b in range(batch_size)], device=device
            )
            current_embedding = self.symbol_embedding(current)

            if self.m_h_0_type == "x_embedding":
                e = torch.cat([current_embedding, m_input_embedding], dim=1)
            elif self.m_h_0_type == "none":
                e = torch.cat([current_embedding, m_input_embedding, x_h_n], dim=1)
            output, (h_n, c_n) = self.message_rnn(e.unsqueeze(1), (h_n, c_n))
            logit = self.message_output_layer(output.squeeze(1))

            logit = logit + self.grammar_mask[current]

            # print("logit", logit)
            # entropy_item = (
            #     -1.0 * torch.softmax(logit, dim=1) * torch.log_softmax(logit, dim=1)
            # )
            # print("entropy_item", entropy_item)
            entropy_item = (
                -1.0
                * torch.where(
                    logit != -float("inf"),
                    torch.softmax(logit, dim=1),
                    (logit != -float("inf")).float(),
                )
                * torch.where(
                    logit != -float("inf"),
                    torch.log_softmax(logit, dim=1),
                    (logit != -float("inf")).float(),
                )
            ).sum(dim=1)
            # entropy_item = entropy_item * (logit != -float("inf")).float()
            # print("entropy_item", entropy_item)
            # entropy_item = entropy_item.sum(dim=1)
            # print("entropy_item", entropy_item)

            next_sample = torch.nn.functional.gumbel_softmax(
                logit, tau=1, hard=True, dim=1
            )
            m_input_embedding = self.message_input_embedding(next_sample)
            for b in range(batch_size):
                output_idx = next_sample[b].argmax(dim=0).item()
                if self._is_token(output_idx):
                    m_history_list[b].append(output_idx)
                    one_hot_m_history_list[b].append(
                        next_sample[b, : self.message_vocab_size]
                    )
                    entropy[b] += entropy_item[b]
                else:
                    rule = self._get_rule(output_idx)
                    for symbol in reversed(rule):
                        if symbol == "[PAD_RULE]":
                            # symbol_idx = self.PAD_RULE_INPUT_INDEX
                            pass
                        else:
                            symbol_idx = self.symbol2id[symbol]
                            stack[b].append(symbol_idx)
                            entropy[b] += entropy_item[b]

            if all([len(stack[b]) == 0 for b in range(batch_size)]):
                break
            elif any(
                [
                    len(m_history_list[b]) >= self.message_max_seq_len - 2
                    for b in range(len(m_history_list))
                ]
            ):
                break
            else:
                for b in range(batch_size):
                    if len(stack[b]) == 0:
                        stack[b].append(self.PAD_RULE_INPUT_INDEX)

        m_padding_mask_list = [
            torch.tensor([False] * (len(m_history_list[b]) + 2))
            for b in range(len(m_history_list))
        ]
        m_padding_mask = torch.nn.utils.rnn.pad_sequence(
            m_padding_mask_list, batch_first=True, padding_value=True
        )
        m_padding_mask = m_padding_mask.to(device)

        m_history_list = [
            torch.tensor(
                [BOS_INDEX] + m_history_list[b] + [EOS_INDEX], dtype=torch.long
            )
            for b in range(len(m_history_list))
        ]
        m_history = torch.nn.utils.rnn.pad_sequence(
            m_history_list, batch_first=True, padding_value=PAD_INDEX
        )
        m_history = m_history.to(device)
        bos_one_hot = torch.zeros(
            self.message_vocab_size, dtype=torch.float, device=device
        )
        bos_one_hot[BOS_INDEX] = 1.0
        eos_one_hot = torch.zeros(
            self.message_vocab_size, dtype=torch.float, device=device
        )
        eos_one_hot[EOS_INDEX] = 1.0

        one_hot_m_history_list = [
            torch.stack([bos_one_hot] + l + [eos_one_hot])
            for l in one_hot_m_history_list
        ]
        one_hot_m_history = torch.nn.utils.rnn.pad_sequence(
            one_hot_m_history_list, batch_first=True
        )

        assert torch.all(one_hot_m_history[:, :, PAD_INDEX] == 0).item()
        return m_history, m_padding_mask, one_hot_m_history, entropy

    def _is_token(self, output_idx: int) -> bool:
        assert isinstance(self.message_grammar, LL1Grammar)

        return output_idx < self.message_vocab_size

    def _get_rule(self, output_idx: int) -> list[int]:
        assert isinstance(self.message_grammar, LL1Grammar)

        if output_idx == self.PAD_RULE_OUTPUT_INDEX:
            return ["[PAD_RULE]"]

        rule_idx = output_idx - self.message_vocab_size
        assert 0 <= rule_idx and rule_idx < len(self.rule_id2rule)

        return self.rule_id2rule[rule_idx][1]
