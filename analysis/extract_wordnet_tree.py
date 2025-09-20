# import nltk

# nltk.download("wordnet")

from nltk.corpus import wordnet as wn


class Node:
    def __init__(self, name):
        self.name = name
        self.children = []
        self.descendants_category_count = 0
        self.descendants_token_count = 0
        self.multi_hypernym = []

    def add_child(self, child_node):
        if child_node not in self.children:
            self.children.append(child_node)

    def rm_child(self, child_node):
        if child_node in self.children:
            self.children.remove(child_node)

    def __repr__(self):
        return self.name


# ノードをキャッシュする辞書
node_map = {}


def get_or_create_node(synset):
    global node_map
    key = synset.name()
    if key not in node_map:
        node_map[key] = Node(key)
    return node_map[key]


def update_node(node):
    global node_map
    key = node.name
    if key not in node_map:
        return
    node_map[key] = node


root = Node("Root")


def extract_wordnet_tree():
    """
    Extracts the WordNet tree structure from the NLTK WordNet corpus.
    Returns a root node containing the entire tree.
    """

    for synset in wn.all_synsets():
        if synset.pos() != "n":  # Only process noun synsets
            continue

        node = get_or_create_node(synset)

        for hyponym in synset.hyponyms():
            hyponym_node = get_or_create_node(hyponym)
            node.add_child(hyponym_node)

        if len(synset.hypernyms()) == 0 and len(synset.instance_hypernyms()) == 0:
            root.add_child(node)
        elif len(synset.hypernyms() + synset.instance_hypernyms()) == 1:
            continue
        else:
            node.multi_hypernym = [
                s.name() for s in synset.hypernyms() + synset.instance_hypernyms()
            ]

    return root


def count_descendants(node, level=0, max_level=100):
    """
    Counts the number of descendants for each node in the tree.
    """

    if not node.children:
        node.descendants_category_count = 0
        node.descendants_token_count = 0
        return 0, 1

    category_count = 0
    token_count = 0
    for child in node.children:
        a, b = count_descendants(child, level + 1, max_level)
        category_count += a
        token_count += b

    node.descendants_category_count = category_count
    node.descendants_token_count = token_count
    update_node(node)
    if level > max_level:
        return 0, token_count
    else:
        return category_count + 1, token_count


def sort_children(node, by="category"):
    """
    Sorts the children of a node based on their names.
    """
    if by == "category":
        node.children.sort(key=lambda x: x.descendants_category_count, reverse=True)
    elif by == "token":
        node.children.sort(key=lambda x: x.descendants_token_count, reverse=True)

    for child in node.children:
        sort_children(child, by=by)
    update_node(node)


def show_tree(node, level=0, max_level=3):
    """
    Recursively prints the tree structure up to a specified level.
    """
    if level > max_level:
        return

    if node.multi_hypernym:
        print(
            "  " * level
            + node.name
            + f" (Descendant categories: {node.descendants_category_count})"
            + f" (Descendant tokens: {node.descendants_token_count})"
            + f" (Multi-hypernyms: {[show_path_from_root(node_map[n]) for n in node.multi_hypernym]})"
        )
    else:
        if node.children:
            print(
                "  " * level
                + node.name
                + f" (Descendant categories: {node.descendants_category_count})"
                + f" (Descendant tokens: {node.descendants_token_count})"
            )
        else:
            print("  " * level + node.name)
    for child in node.children:
        show_tree(child, level + 1, max_level)


def show_category_count(node):
    """
    Displays the count of categories (children) for each node in the tree.
    """
    if node.children:
        count = 0
        for child in node.children:
            count += show_category_count(child)
        return count + 1
    else:
        return 0


def show_path_from_root(node):
    """
    Displays the path from the root to the given node.
    """
    path = []
    current = node
    while current:
        path.append(current.name)
        parent = None
        for n in node_map.values():
            if current in n.children:
                parent = n
                break
        current = parent
    return " -> ".join(reversed(path))


root = extract_wordnet_tree()
count_descendants(root, max_level=100)
sort_children(root, by="token")

# show_tree(root, max_level=1)
# print(show_category_count(root))
show_tree(get_or_create_node(wn.synset("group.n.01")), max_level=1)
# p, t, k, s, h
# b, d, g, m, n, r
# a, i, u, e, o
# 11 + 5 = 16
# 11 * 5 = 55
# 11 * 11 = 121

# tokenを4096を最大値としてクラス分け
# causal_agentは型として与えるので、agent以下は不採用

# ba entity.n.01 -> physical_entity.n.01 -> object.n.01 -> whole.n.02 -> living_thing.n.01 -> organism.n.01 -> person.n.01 (6266)
# be entity.n.01 -> physical_entity.n.01 -> object.n.01 -> whole.n.02 -> living_thing.n.01 -> organism.n.01 -> plant.n.02 (3908)
# bi entity.n.01 -> physical_entity.n.01 -> object.n.01 -> whole.n.02 -> living_thing.n.01 -> organism.n.01 -> animal.n.01 (3218)
# bu entity.n.01 -> physical_entity.n.01 -> object.n.01 -> whole.n.02 -> living_thing.n.01 ->  * (organism.n.01 -> person.n.01, organism.n.01 -> plant.n.02, organism.n.01 -> animal.n.01 以外) (14074 - 6266 - 3908 - 3218 = 682)
# da entity.n.01 -> physical_entity.n.01 -> object.n.01 -> whole.n.02 -> artifact.n.01 -> instrumentality.n.03 -> device.n.01 (2137)
# de entity.n.01 -> physical_entity.n.01 -> object.n.01 -> whole.n.02 -> artifact.n.01 -> instrumentality.n.03 -> * (device.n.01 以外) (4096 - 2137 = 1959)
# di entity.n.01 -> physical_entity.n.01 -> object.n.01 -> whole.n.02 -> artifact.n.01 -> structure.n.01 (1123)
# du entity.n.01 -> physical_entity.n.01 -> object.n.01 -> whole.n.02 -> artifact.n.01 -> * (instrumentality.n.03, structure.n.01 以外) (9142 - 4625 - 1123 = 3394)
# na entity.n.01 -> physical_entity.n.01 -> object.n.01 -> whole.n.02 -> * (living_thing.n.01, artifact.n.01 以外) (24147 - 14074 - 9142 = 931)
# ne entity.n.01 -> physical_entity.n.01 -> object.n.01 -> * (whole.n.02 以外) (25554 - 24147 = 1407)
# (不採用) (重複) entity.n.01 -> physical_entity.n.01 -> causal_agent.n.01 -> person.n.01 (6266)
# (不採用) (一時的) entity.n.01 -> physical_entity.n.01 -> causal_agent.n.01 -> agent.n.03 (915)　(Multi-hypernyms: ['causal_agent.n.01', 'substance.n.07'])  entity.n.01 -> physical_entity.n.01 -> matter.n.03 -> substance.n.07
# (不採用) entity.n.01 -> physical_entity.n.01 -> causal_agent.n.01 -> * (person.n.01, agent.n.03 以外) (7311 - 6266 - 915 = 130)
# ni entity.n.01 -> physical_entity.n.01 -> matter.n.03 -> substance.n.01 (3045)
# no entity.n.01 -> physical_entity.n.01 -> matter.n.03 -> * (substance.n.01 以外) (6424 - 3045 = 3379)
# nu entity.n.01 -> physical_entity.n.01 -> * (object.n.01, causal_agent.n.01, matter.n.03 以外) (42412 - 25554 - 7311 - 6424 = 3123)
# ga entity.n.01 -> abstraction.n.06 -> psychological_feature.n.01 -> event.n.01 -> act.n.02 -> activity.n.01 (2616)
# ge entity.n.01 -> abstraction.n.06 -> psychological_feature.n.01 -> event.n.01 -> act.n.02 -> * (activity.n.01 以外) (5087 - 2616 = 2471)
# gi entity.n.01 -> abstraction.n.06 -> psychological_feature.n.01 -> event.n.01 -> * (act.n.02 以外) (6565 - 5087 = 1478)
# gu entity.n.01 -> abstraction.n.06 -> psychological_feature.n.01 -> * (event.n.01 以外) (9612 - 6565 = 3047)
# ma entity.n.01 -> abstraction.n.06 -> group.n.01 -> biological_group.n.01 -> taxonomic_group.n.01 -> genus.n.02 (3571)
# mi entity.n.01 -> abstraction.n.06 -> group.n.01 -> biological_group.n.01 -> * (taxonomic_group.n.01 -> genus.n.02 以外) (5381 - 3571 = 1810)
# mu entity.n.01 -> abstraction.n.06 -> group.n.01 -> * (biological_group.n.01 以外) (7518 - 5381 = 2137)
# ra entity.n.01 -> abstraction.n.06 -> attribute.n.02 -> state.n.02 (2929)
# re entity.n.01 -> abstraction.n.06 -> attribute.n.02 -> * (state.n.02 以外) (5455 - 2929 = 2526)
# (不採用) (重複) entity.n.01 -> abstraction.n.06 -> relation.n.01 -> part.n.01 -> substance.n.01 (3045) (Multi-hypernyms: ['part.n.01', 'matter.n.03'])
# ri entity.n.01 -> abstraction.n.06 -> relation.n.01 -> * (part.n.01 -> substance.n.01 以外) (4313 - 3045 = 1268)
# ro entity.n.01 -> abstraction.n.06 -> communication.n.02 (3640)
# ru entity.n.01 -> abstraction.n.06 -> * (psychological_feature.n.01, group.n.01, attribute.n.02, relation.n.01, communication.n.02 以外) (32477 - 9612 - 7518 - 5455 - 4313 - 3640 = 1939)
# (不採用) entity.n.01 -> thing.n.08 (8)

# Multi-hypernyms
# recombinant.n.01 (Descendant categories: 0) (Descendant tokens: 0) (Multi-hypernyms: ['entity.n.01 -> physical_entity.n.01 -> object.n.01 -> whole.n.02 -> living_thing.n.01 -> organism.n.01', 'entity.n.01 -> physical_entity.n.01 -> object.n.01 -> whole.n.02 -> living_thing.n.01 -> cell.n.02'])
# microflora.n.01 (Descendant categories: 0) (Descendant tokens: 0) (Multi-hypernyms: ['entity.n.01 -> physical_entity.n.01 -> object.n.01 -> whole.n.02 -> living_thing.n.01 -> organism.n.01 -> microorganism.n.01', 'entity.n.01 -> physical_entity.n.01 -> object.n.01 -> whole.n.02 -> living_thing.n.01 -> organism.n.01 -> plant.n.02'])
# jungle_gym.n.01 (Descendant categories: 0) (Descendant tokens: 0) (Multi-hypernyms: ['entity.n.01 -> physical_entity.n.01 -> object.n.01 -> whole.n.02 -> artifact.n.01 -> structure.n.01', 'entity.n.01 -> physical_entity.n.01 -> object.n.01 -> whole.n.02 -> artifact.n.01 -> plaything.n.01'])
# agent.n.03 (Descendant categories: 254) (Descendant tokens: 915) (Multi-hypernyms: ['entity.n.01 -> physical_entity.n.01 -> causal_agent.n.01', 'entity.n.01 -> physical_entity.n.01 -> matter.n.03 -> substance.n.07'])
# world.n.08 (Descendant categories: 0) (Descendant tokens: 0) (Multi-hypernyms: ['entity.n.01 -> abstraction.n.06 -> group.n.01', 'entity.n.01 -> physical_entity.n.01 -> object.n.01 -> whole.n.02 -> living_thing.n.01 -> organism.n.01 -> animal.n.01 -> chordate.n.01 -> vertebrate.n.01 -> mammal.n.01 -> placental.n.01 -> primate.n.02 -> hominid.n.01 -> homo.n.02'])
# mathematical_space.n.01 (Descendant categories: 1) (Descendant tokens: 5) (Multi-hypernyms: ['entity.n.01 -> abstraction.n.06 -> set.n.02', 'entity.n.01 -> abstraction.n.06 -> attribute.n.02 -> space.n.01'])
# outer_space.n.01 (Descendant categories: 0) (Descendant tokens: 0) (Multi-hypernyms: ['entity.n.01 -> abstraction.n.06 -> attribute.n.02 -> space.n.01', 'entity.n.01 -> physical_entity.n.01 -> object.n.01 -> location.n.01'])
