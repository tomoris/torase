import torch
from torch import nn
from torch.utils.data import Dataset, DataLoader

from compiler.ll_one.grammar import Grammar as LL1Grammar
from signaling_game.sender import Sender
from signaling_game.receiver import Receiver


class SignalingGame(nn.Module):
    def __init__(self, sender: Sender, receiver: Receiver):
        super(SignalingGame, self).__init__()
        self.sender = sender
        self.receiver = receiver


class MyDataset(Dataset):
    def __init__(self, corpus):
        self.corpus = corpus

    def __len__(self):
        return len(self.corpus)

    def __getitem__(self, idx):
        return self.corpus[idx]


def collate_fn(batch):
    for i in range(len(batch)):
        batch[i] = torch.tensor(batch[i])
    return torch.nn.utils.rnn.pad_sequence(batch, batch_first=True)


def main():
    corpus = [
        ["[BOS]", "this", "is", "test", "sentence", "one", "[EOS]"],
        ["[BOS]", "これは", "test", "文", "です", "one", "。", "[EOS]"],
    ]
    word2id = {
        "[PAD]": 0,
        "[BOS]": 1,
        "[EOS]": 2,
        "this": 3,
        "is": 4,
        "test": 5,
        "sentence": 6,
        "one": 7,
        "これは": 8,
        "文": 9,
        "です": 10,
        "。": 11,
    }
    id2word = {v: k for k, v in word2id.items()}
    corpus = [[word2id[word] for word in sentence] for sentence in corpus]
    dataset = MyDataset(corpus)
    dataloader = DataLoader(dataset, batch_size=2, shuffle=True, collate_fn=collate_fn)

    rules = {
        "S": [["subject", "verb_phrase", "object"]],
        "subject": [["noun_phrase"], ["S_preposition", "S"]],
        "verb_phrase": [["verb"], ["auxiliary_verb", "verb"]],
        "object": [["noun_phrase"], ["S_preposition", "S"]],
        "noun_phrase": [["compound_preposition", "noun", "noun_phrase"], ["noun"]],
        "noun": [["nominalization_preposition", "verb"]],
    }
    terminals = {
        "compound_preposition",
        "nominalization_preposition",
        "S_preposition",
        "verb",
        "auxiliary_verb",
    }
    # ll1_grammar = LL1Grammar(rules, terminals)
    ll1_grammar = None
    terminal2vocab_size = {
        "compound_preposition": 1,
        "nominalization_preposition": 1,
        "S_preposition": 1,
        "verb": 10,
        "auxiliary_verb": 5,
    }

    sender = Sender(
        input_vocab_size=len(word2id),
        hidden_dim=10,
        message_vocab_size=18,
        message_max_seq_len=100,
        m_h_0_type="x_embedding",
        message_grammar=ll1_grammar,
        terminal2vocab_size=terminal2vocab_size,
    )
    receiver = Receiver(
        input_vocab_size=len(word2id),
        hidden_dim=10,
        message_vocab_size=18,
        x_max_seq_len=10,
    )
    model = SignalingGame(sender, receiver)

    optimizer = torch.optim.Adam(model.parameters())

    for _ in range(10000):
        for batch in dataloader:
            optimizer.zero_grad()

            x = batch
            x_pad_mask = x != 0
            message, message_pad_mask, one_hot_message, entropy = model.sender(
                x, x_pad_mask
            )
            loss = model.receiver(message, one_hot_message, message_pad_mask, x)
            print(entropy)
            loss = loss - entropy.mean()
            loss.backward()
            optimizer.step()

            predicted_sentence = receiver.predict(
                message, one_hot_message, message_pad_mask
            )
            print(loss)
            print(message)
            print(
                [id2word[id.item()] for id in x[0]],
                [id2word[id.item()] for id in predicted_sentence[0]],
            )
            print(
                [id2word[id.item()] for id in x[1]],
                [id2word[id.item()] for id in predicted_sentence[1]],
            )
            print()


if __name__ == "__main__":
    main()
