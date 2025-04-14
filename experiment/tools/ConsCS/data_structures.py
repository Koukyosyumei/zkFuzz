from misc import * 
    
class UnionFind:
    def __init__(self):
        self.parent = {}
        
    def check_element(self, x):
        if x not in self.parent:
            self.add(x)
    
    def find(self, x):
        self.check_element(x)
        path = []
        root = x
        while self.parent[root] != root:
            path.append(root)
            root = self.parent[root]
        # path compression
        for node in path:
            self.parent[node] = root
        return root
    
    def union(self, x, y):
        rootX = self.find(x)
        rootY = self.find(y)
        if rootX != rootY:
            self.parent[rootY] = rootX
    
    def add(self, x):
        if x not in self.parent:
            self.parent[x] = x

            
    def get_distinct_groups(self):
        groups = {}
        for element in self.parent:
            root = self.find(element)
            if root not in groups:
                groups[root] = set()
            groups[root].add(element)
        return list(groups.values())
            
    def get_group_members(self, x):
        self.check_element(x)
        root = self.find(x)
        return {element for element in self.parent if self.find(element) == root}
            
        
        
class Edge:
    next_edge_id = 0
            

class Hypothesis_Edge(Edge):
    def __init__(self, target, condition, cur_cons_id):
        self.target = target
        assert isinstance(condition, dict)
        self.condition = condition
        self.cur_cons_id = cur_cons_id
        self.edge_id = Edge.next_edge_id
        Edge.next_edge_id += 1

    def __repr__(self):
        return f"Edge(target={self.target}, condition={self.condition}, id={self.edge_id}, cur_cons_id={self.cur_cons_id})"

    
class BPG_Edge(Edge):
    def __init__(self, from_node, to_node, cur_cons_id, edge_condition):
        self.from_node = from_node
        self.to_node = to_node
        self.cur_cons_id = cur_cons_id
        self.edge_id = Edge.next_edge_id
        self.edge_condition = edge_condition
        Edge.next_edge_id += 1

    def __repr__(self):
        return f"Edge(from_node={self.from_node}, to_node={self.to_node}, id={self.edge_id}, edge_condition={self.edge_condition}, cur_cons_id={self.cur_cons_id})"
    


    
class Hypothesis_Assumptions:
    def __init__(self):
        self.assumptions = dict()
        
class BPG():
    def __init__(self):
        self.graph = dict()
        self.invariant_properties = set()
        
