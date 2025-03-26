import torch
from torch import nn, Tensor


PAD_INDEX = 0
BOS_INDEX = 1
EOS_INDEX = 2
SPECIAL_TOKENS_NUM = 3


class Receiver(nn.Module):
    def __init__(
        self,
        input_vocab_size: int,
        hidden_dim: int,
        message_vocab_size: int,
        x_max_seq_len: int,
    ):
        super(Receiver, self).__init__()

        self.input_vocab_size = input_vocab_size
        self.hidden_dim = hidden_dim
        self.message_vocab_size = message_vocab_size + SPECIAL_TOKENS_NUM
        self.x_max_seq_len = x_max_seq_len

        self.x_token_embedding = nn.Embedding(
            input_vocab_size, hidden_dim, padding_idx=PAD_INDEX
        )
        self.x_rnn = nn.LSTM(hidden_dim, hidden_dim, batch_first=True)
        self.x_output_layer = nn.Linear(hidden_dim, input_vocab_size)

        self.message_token_embedding = nn.Linear(self.message_vocab_size, hidden_dim)
        self.message_rnn = nn.LSTM(hidden_dim, hidden_dim, batch_first=True)
        self.message_output_layer = nn.Linear(hidden_dim, self.message_vocab_size)

        self.loss_fn = nn.CrossEntropyLoss(ignore_index=PAD_INDEX, reduction="none")

    def forward(
        self,
        message: Tensor,
        one_hot_message: Tensor,
        message_pad_mask: Tensor,
        x: Tensor,
    ) -> Tensor:
        (h_n, c_n), surprisal = self._calc_message_language_model_state_and_surprisal(
            message, one_hot_message, message_pad_mask
        )

        x_token_embeddings = self.x_token_embedding(x)
        hidden, _ = self.x_rnn(x_token_embeddings, (h_n, c_n))
        output = self.x_output_layer(hidden[:, :-1, :])
        reconstruction_loss = self.loss_fn(
            output.resize(x.size(0) * (x.size(1) - 1), self.input_vocab_size),
            x[:, 1:].resize(x.size(0) * (x.size(1) - 1)),
        )
        reconstruction_loss = reconstruction_loss.resize(x.size(0), x.size(1) - 1).sum(
            dim=1
        )

        return (reconstruction_loss + surprisal).mean()

    def _calc_message_language_model_state_and_surprisal(
        self, message: Tensor, one_hot_message: Tensor, message_pad_mask: Tensor
    ) -> tuple[tuple[Tensor, Tensor], Tensor]:
        assert len(one_hot_message.size()) == 3
        assert one_hot_message.size(2) == self.message_vocab_size
        assert torch.all(one_hot_message[:, :, PAD_INDEX] == 0).item()
        assert message.size(0) == one_hot_message.size(0)
        assert message.size(1) == one_hot_message.size(1)
        assert message.size(0) == message_pad_mask.size(0)
        assert message.size(1) == message_pad_mask.size(1)

        m_token_embeddings = self.message_token_embedding(one_hot_message)
        batch_size = one_hot_message.size(0)
        h_n = torch.zeros(
            (1, batch_size, self.hidden_dim), device=m_token_embeddings.device
        )
        c_n = torch.zeros(
            (1, batch_size, self.hidden_dim), device=m_token_embeddings.device
        )
        final_h_n = torch.zeros(
            (1, batch_size, self.hidden_dim), device=m_token_embeddings.device
        )
        final_c_n = torch.zeros(
            (1, batch_size, self.hidden_dim), device=m_token_embeddings.device
        )
        final_n_flags = [False] * batch_size
        log_prob_list = []
        pad_mask = torch.ones(
            (batch_size, self.message_vocab_size),
            dtype=torch.float,
            device=message.device,
        )
        pad_mask[:, PAD_INDEX] = 0.0
        for n in range(one_hot_message.size(1)):
            m_token_embedding = m_token_embeddings[:, n, :]
            output, (h_n, c_n) = self.message_rnn(
                m_token_embedding.unsqueeze(1), (h_n, c_n)
            )
            log_prob = torch.log_softmax(
                self.message_output_layer(output.squeeze(1)), dim=1
            )
            log_prob = log_prob * pad_mask
            log_prob_list.append(
                torch.stack([p[m] for p, m in zip(log_prob, message[:, n])])
            )
            for b in range(batch_size):
                if message_pad_mask[b, n].item():
                    if not final_n_flags[b]:
                        final_h_n[0, b] = h_n[0, b]
                        final_c_n[0, b] = c_n[0, b]
                        final_n_flags[b] = True
        # paddingがない場合は最後の隠れ状態を使う
        for b in range(batch_size):
            if not message_pad_mask[b, -1].item():
                final_h_n[0, b] = h_n[0, b]
                final_c_n[0, b] = c_n[0, b]
                final_n_flags[b] = True
        assert all(final_n_flags)

        log_prob = torch.stack(log_prob_list, dim=0).T
        assert log_prob.size() == (batch_size, message.size(1))
        return (final_h_n, final_c_n), -torch.sum(log_prob, dim=1)

    def predict(
        self, message: Tensor, one_hot_message: Tensor, message_pad_mask: Tensor
    ) -> Tensor:
        (h_n, c_n), _ = self._calc_message_language_model_state_and_surprisal(
            message, one_hot_message, message_pad_mask
        )

        bos_token = torch.ones(
            (message.size(0), 1),
            dtype=torch.long,
            device=message.device,
        )
        bos_token = bos_token * BOS_INDEX
        x_token_embedding = self.x_token_embedding(bos_token)
        x_history = (
            torch.ones((message.size(0), 1), dtype=torch.long, device=message.device)
            * BOS_INDEX
        )
        while True:
            _, (h_n, c_n) = self.x_rnn(x_token_embedding, (h_n, c_n))
            output = self.x_output_layer(h_n.squeeze(0))
            x_token_embedding, x_history = self._generate_next_token(output, x_history)

            if (
                torch.all(
                    (x_history[:, -1] == EOS_INDEX) | (x_history[:, -1] == PAD_INDEX)
                ).item()
                or x_history.size(1) >= self.x_max_seq_len
            ):
                break

        return x_history

    def _generate_next_token(
        self, output: Tensor, x_history: Tensor
    ) -> tuple[Tensor, Tensor]:
        output[:, PAD_INDEX] = -float("inf")
        next_mask = (x_history[:, -1] == EOS_INDEX) | (x_history[:, -1] == PAD_INDEX)
        next_mask = next_mask.unsqueeze(1)
        pad_token = torch.ones(
            (output.size(0), 1),
            dtype=torch.long,
            device=output.device,
        )
        pad_token = pad_token * PAD_INDEX
        next_token = torch.where(
            next_mask, pad_token, torch.argmax(output, dim=1, keepdim=True)
        )
        x_history = torch.cat([x_history, next_token], dim=1)
        x_token_embedding = self.x_token_embedding(next_token)

        return x_token_embedding, x_history
