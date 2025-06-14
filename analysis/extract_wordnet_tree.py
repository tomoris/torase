# import nltk
# nltk.download("wordnet")

# from nltk.corpus import wordnet as wn


# class Node:
#     def __init__(self, name):
#         self.name = name
#         self.children = []
#         self.descendants_count = 0
#         self.multi_hypernym = []

#     def add_child(self, child_node):
#         if child_node not in self.children:
#             self.children.append(child_node)

#     def rm_child(self, child_node):
#         if child_node in self.children:
#             self.children.remove(child_node)

#     def __repr__(self):
#         return self.name


# # ノードをキャッシュする辞書
# node_map = {}


# def get_or_create_node(synset):
#     key = synset.name()
#     if key not in node_map:
#         node_map[key] = Node(key)
#     return node_map[key]


# root = Node("Root")


# def extract_wordnet_tree():
#     """
#     Extracts the WordNet tree structure from the NLTK WordNet corpus.
#     Returns a root node containing the entire tree.
#     """

#     for synset in wn.all_synsets():
#         if synset.pos() != "n":  # Only process noun synsets
#             continue

#         node = get_or_create_node(synset)

#         for hyponym in synset.hyponyms():
#             hyponym_node = get_or_create_node(hyponym)
#             node.add_child(hyponym_node)

#         if len(synset.hypernyms()) == 0 and len(synset.instance_hypernyms()) == 0:
#             root.add_child(node)
#         elif len(synset.hypernyms() + synset.instance_hypernyms()) == 1:
#             continue
#         else:
#             node.multi_hypernym = [
#                 s.name() for s in synset.hypernyms() + synset.instance_hypernyms()
#             ]

#     return root


# def count_descendants(node):
#     """
#     Counts the number of descendants for each node in the tree.
#     """
#     if not node.children:
#         node.descendants_count = 0
#         return 1

#     count = 0
#     for child in node.children:
#         count += count_descendants(child)

#     node.descendants_count = count
#     return count + 1


# def sort_children(node):
#     """
#     Sorts the children of a node based on their names.
#     """
#     node.children.sort(key=lambda x: x.descendants_count, reverse=True)
#     for child in node.children:
#         sort_children(child)


# def show_tree(node, level=0, max_level=3):
#     """
#     Recursively prints the tree structure up to a specified level.
#     """
#     if level > max_level:
#         return

#     if node.multi_hypernym:
#         print(
#             "  " * level
#             + node.name
#             + f" (Descendants: {node.descendants_count})"
#             + f" (Multi-hypernyms: {node.multi_hypernym})"
#         )
#     else:
#         if not node.children:
#             print(
#                 "  " * level + node.name + f" (Descendants: {node.descendants_count})"
#             )
#     for child in node.children:
#         show_tree(child, level + 1, max_level)


# root = extract_wordnet_tree()
# count_descendants(root)
# sort_children(root)

# show_tree(root, max_level=4)
