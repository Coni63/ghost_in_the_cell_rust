import sys
import math
from collections import defaultdict, deque
from time import time
from itertools import permutations

def dijkstra(graph, initial):
    visited = {initial: 0}
    path = {}

    nodes = set(graph.nodes)

    while nodes:
        min_node = None
        for node in nodes:
            if node in visited:
                if min_node is None:
                    min_node = node
                elif visited[node] < visited[min_node]:
                    min_node = node
        if min_node is None:
            break

        nodes.remove(min_node)
        current_weight = visited[min_node]

        for edge in graph.edges[min_node]:
            try:
                weight = current_weight + graph.distances[(min_node, edge)]
            except:
                continue
            if edge not in visited or weight < visited[edge]:
                visited[edge] = weight
                path[edge] = min_node

    return visited, path

def shortest_path(graph, origin, destination):
    visited, paths = dijkstra(graph, origin)
    full_path = deque()
    _destination = paths[destination]
    #dist = 0

    while _destination != origin:
        full_path.appendleft(_destination)
        _destination = paths[_destination]
        #dist += graph.distances[(paths[_destination],destination)]

    full_path.appendleft(origin)
    full_path.append(destination)

    return visited[destination], list(full_path)

class Strategy:
    def __init__(self):
        self.bomb = 2
        self.multi_output = True
        self.scores = [0, 0] #moi et ennemie
        self.productivity = [0, 0] #moi et ennemie
        self.base_controlled = [0, 0, 0]
        self.loop = 0
        self.main_queue = []
        self.command = []
        self.permutation = list(permutations(G.nodes, 2))
        print(self.permutation, file=sys.stderr)
        
    def new_turn(self):
        Factory.my_bases = {}
        Factory.opponents_bases = {}
        Factory.free_bases = {}
        Cyborg.instances = {}
        Cyborg.my_robots = {}
        Cyborg.ennemies = {}
        Cyborg.incoming_attacks = 0
        Bomb.my_bomb = []
        Bomb.enemy_ongoing_bomb = []
        self.scores = [0, 0] #moi et ennemie
        self.productivity = [0, 0]#moi et ennemie
        self.base_controlled = [0, 0, 0]
        self.main_queue = []
        self.command = []

    def clear_risk_of_impact(self):
        for i in range(len(Bomb.enemy_bomb)):
            if not Bomb.enemy_bomb[i] in Bomb.enemy_ongoing_bomb: #si la bombe a impacté
                 for each_bases in Factory.my_bases.values():
                    each_bases.risk[i] = 0

    def defend_my_bases(self):
        ordered_bases = sorted(Factory.my_bases.values(), key=lambda obj:obj.defense, reverse=True)
        #print([a.ID for a in ordered_bases], file=sys.stderr)
        for each_bases in ordered_bases:
            if each_bases.robot_available > 1:
                for each_other_bases in ordered_bases:
                    if each_other_bases != each_bases: 
                        if each_other_bases.robot_available < 2:
                            support = min( each_bases.robot_available, 2-each_other_bases.robot_available)
                            each_bases.robot_available -= support
                            self.main_queue.insert(0, (each_bases.ID, each_other_bases.ID, support))

    def set_availability(self):
        for each in Factory.instances.values():
            each.robot_available = each.defense+each.ongoing_support-each.ongoing_ennemies
    
    def expand_fast(self):
        print(list(Factory.my_bases.values()), file=sys.stderr)
        ordered_factory = list(sorted(Factory.my_bases.values(), key = lambda x : x.robot_available))
        for factory in ordered_factory:
            list_of_voisin = [Factory.instances[a] for a in G.edges[factory.ID]]
            list_of_voisin.sort(key = lambda x : G.optimised_distances[(factory.ID, x.ID)])
            list_of_voisin.sort(key = lambda x : x.team, reverse = True)
            safety_base_vide = 1
            safety_base_ennemie = 5
            for each_voisin in list_of_voisin:
                if each_voisin.prod >= 1:
                    opti_path = each_voisin.ID#G.optimised_path[(factory.ID,each_voisin.ID)]
                    if each_voisin.team == 0:
                        #print("path 1", file=sys.stderr)
                        qte = each_voisin.defense + each_voisin.ongoing_ennemies - each_voisin.ongoing_support
                        if factory.robot_available >= qte + safety_base_vide:
                            self.main_queue.append((factory.ID, opti_path, qte + safety_base_vide))
                            factory.robot_available -= qte + safety_base_vide
                            each_voisin.ongoing_support += qte + safety_base_vide
                    elif each_voisin.team == 1:
                        #print("path 2", file=sys.stderr)
                        if each_voisin.robot_available < 0 and factory.robot_available > 0:
                            support = min(1-each_voisin.robot_available, factory.robot_available)
                            self.main_queue.append((factory.ID, opti_path, support))
                            factory.robot_available -= support
                            each_voisin.ongoing_support += support
                    else:
                        pass
                        """
                        qte = each_voisin.defense + each_voisin.ongoing_ennemies + G.optimised_distances[(factory.ID, each_voisin.ID)] * each_voisin.prod
                        print("path 3", file=sys.stderr)
                        if factory.robot_available >= qte + safety_base_ennemie:
                            self.main_queue.append((factory.ID, opti_path, qte + safety_base_ennemie))
                            factory.robot_available -= qte + safety_base_ennemie
                        """
                        
            
    def check_best_target(self):
        maxi = 11 #ne pas envoyer avant 11 each.defense > maxi and
        target = None
        base = None
        for each in Factory.opponents_bases.values():
            if each.prod >= 2 and each.ID not in Bomb.my_bomb:
                target = each.ID
        
        if not target is None:
            mini = 21
            for each in Factory.my_bases.values():
                if G.distances[(each.ID, target)] < mini:
                    base = each.ID
        
        return target, base
                
    def we_dominate(self):
        if (
            (self.base_controlled[0] > self.base_controlled[1]) or 
            (self.base_controlled[0] == self.base_controlled[1] and self.productivity[0] > self.productivity[1]) or 
            (self.base_controlled[0] == self.base_controlled[1] and self.productivity[0] == self.productivity[1] and self.scores[0] > self.scores[1])
            ):
            return True
        else:
            return False
    
    def queue_to_command(self):
        if self.main_queue == []:
            self.command.append("WAIT")
        else:
            self.command += ["MOVE %s %s %s"% (each[0], each[1], each[2]) for each in self.main_queue]
    
    def check_for_base_update(self):
        update_list = []
        for each in Factory.my_bases.values():
            if each.prod < 3 and each.robot_available > 10 and self.loop < 300:
                update_list.append(each.ID)
        return update_list    
    
    def create_strategy(self):
        print(self.main_queue,file=sys.stderr)
        self.expand_fast()
        print(self.main_queue,file=sys.stderr)
        
        
        for each_base in Factory.my_bases.values():
            if each_base.risk[0] == 1 or each_base.risk[1] == 1:
                self.main_queue.append(each_base.leave())   
        
        print(self.main_queue,file=sys.stderr)
        self.queue_to_command()
        print(self.command,file=sys.stderr)        
                
        if self.bomb > 0: # and loop > 30:
            target, base = self.check_best_target()
            if not target is None:
                self.command.append("BOMB %s %s"%(base, target))
                self.bomb -= 1
        
        if self.loop > 20: #self.scores[0] > self.scores[1] + 20:        
            base_to_update = self.check_for_base_update()
            if base_to_update != []:
                for each in base_to_update:
                    self.command.append("INC %s" %(each))
        
        print(self.command,file=sys.stderr) 
        
        """               
        if Cyborg.incoming_attacks > 0:
            for each_base in Factory.my_bases.values():
                if each_base.prod == 0:
                    for other_base in G.optimised_voisin[each_base.ID]:
                        if Factory.instances[other_base].team == 1 and Factory.instances[other_base].prod > 0:
                            self.command.append("MOVE %s %s %s" %(each_base.ID, other_base, each_base.defense))
        else:
            for each_base in Factory.my_bases.values():
                if each_base.prod == 0:
                    each_base.ask_for_robot = 20 - each_base.defense - each_base.ongoing_support + each_base.ongoing_ennemies
                    list_of_voisin = [Factory.instances[a] for a in G.nodes[each_base.ID]]
                    list_of_voisin.filter(lambda x : x.team == 1)
                    list_of_voisin.sort(key = lambda x : x.factory.robot_available, reverse = True)
                    for voisin in list_of_voisin:
                        support = min(voisin.robot_available, each_base.ask_for_robot)
                        each_bases.ask_for_robot -= support
                        self.main_queue.insert(0, (voisin.ID, each_base.ID, support))
        """     
        
    
class Graph:
    def __init__(self):
        self.nodes = set()
        self.edges = defaultdict(list)
        self.distances = {}
        self.links = 0
        self.optimised_path = {}
        self.optimised_distances = {}
        self.optimised_voisin = defaultdict(list)

    def ajouteSommet(self, sommet):
        if sommet not in self.nodes:
            self.nodes.add(sommet)

    def ajouteArrete(self, sommet, sommetVoisin, dist=1):
        if sommet != sommetVoisin:
            self.edges[sommetVoisin].append(sommet) 
            self.edges[sommet].append(sommetVoisin)
            self.distances[(sommet, sommetVoisin)] = dist
            self.distances[(sommetVoisin, sommet)] = dist
            self.links += 1
    
    def show(self):
        print(self.__dict__, file=sys.stderr)
        #for key, value in self.distances.items():
        #    print("%s -> %s : %s tour" % (key[0], key[1], value) ,file=sys.stderr)
            
       
class Factory:
    instances = {}
    my_bases = {}
    opponents_bases = {}
    free_bases = {}
    def __init__(self, ID, team, defense, prod):
        if team == 1 :
            self.my_bases[ID] = self
            S.productivity[0] += prod
            S.scores[0] += defense
            S.base_controlled[0] += 1
        elif team == -1:
            self.opponents_bases[ID] = self
            S.productivity[1] += prod
            S.scores[1] += defense
            S.base_controlled[1] += 1
        else:
            self.free_bases[ID] = self
            S.base_controlled[2] += 1
        self.ID = ID
        self.team = team
        self.defense = defense
        self.prod = prod
        self.instances[ID]=self
        self.ongoing_support = 0
        self.ongoing_ennemies = 0
        self.risk = [0, 0]
        self.robot_available = 0
                
    def update(self, team, defense, prod):
        if team == 1 :
            self.my_bases[self.ID] = self
            S.productivity[0] += prod
            S.scores[0] += defense
            S.base_controlled[0] += 1
        elif team == -1:
            self.opponents_bases[self.ID] = self
            S.productivity[1] += prod
            S.scores[1] += defense
            S.base_controlled[1] += 1
        else:
            self.free_bases[self.ID] =self
            S.base_controlled[2] += 1
        self.team = team
        self.defense = defense
        self.prod = prod
        self.ongoing_support = 0
        self.ongoing_ennemies = 0
        self.robot_available = 0
        self.risk[0] = max(0, self.risk[0]-1)
        self.risk[1] = max(0, self.risk[1]-1)
    
    def leave(self):
        closest = 21
        target = None
        qte = self.defense
        for each_other_base in Factory.my_bases.values():
            if each_other_base != self:
                d = G.distances[(self.ID, each_other_base.ID)]
                if d < closest:
                    closest = d
                    target = each_other_base.ID
        if target is None:
            smallest = 10000
            for other_base in Factory.opponents_bases.values():
                if other_base.defense < smallest:
                    smallest = other_base.defense
                    target = other_base.ID
        return (self.ID, target, qte+5)
    
    def show(self):
        print(self.__dict__, file=sys.stderr)
                  
class Cyborg:
    instances = {}
    my_robots = {}
    ennemies = {}
    limit_turn = 10
    incoming_attacks = 0
    def __init__(self, ID, team, s, e, num, t):
        if team == 1 :
            self.my_robots[ID] = self
            self.team = "me"
            S.scores[0] += num
            if t < self.limit_turn:
                Factory.instances[e].ongoing_support += num
        else:
            self.ennemies[ID] = self
            self.team = "opponent"
            S.scores[1] += num
            if t < self.limit_turn:
                Factory.instances[e].ongoing_ennemies += num   
            if Factory.instances[e].team == 1:
                self.incoming_attacks += 1
                
        self.ID = ID
        self.s_factory = s
        self.e_factory = e
        self.troop_size = num
        self.remaining_turn = t
        self.instances[ID] = self        

    def show(self):
        print(self.__dict__, file=sys.stderr)

class Bomb:
    instances = {}
    my_bomb = []
    enemy_bomb = [] #ID des bombes ennemies pour savoir si c'est une nouvelle bombe
    enemy_ongoing_bomb = []
    def __init__(self, ID, t, s, e):
        self.ID = ID
        self.team = t
        self.start = s
        self.end = e
        self.instances[ID] = self
        if t == 1:
            self.my_bomb.append(e)
        if t == -1:
            self.enemy_ongoing_bomb.append(ID)
            if not ID in self.enemy_bomb:
                self.enemy_bomb.append(ID)
                bomb_number = len(self.enemy_bomb)-1
                for each_bases in Factory.my_bases.values():
                    if s != each_bases.ID:
                        d = G.distances[(s, each_bases.ID)]
                        each_bases.risk[bomb_number] = d
    
    def show(self):
        print(self.__dict__, file=sys.stderr)

G = Graph()


factory_count = int(input())  # the number of factories
link_count = int(input())  # the number of links between factories
for i in range(link_count):
    factory_1, factory_2, distance = [int(j) for j in input().split()]
    G.ajouteSommet(factory_1)
    G.ajouteSommet(factory_2)
    G.ajouteArrete(factory_1, factory_2, distance)

S = Strategy()

for each_input in G.nodes:
    for each_output in G.nodes:
        if each_input != each_output:
            dist, path = shortest_path(G, each_input, each_output)
            #print((each_input, each_output), dist, path, file=sys.stderr)
            G.optimised_path[(each_input, each_output)] = path[1]
            G.optimised_distances[(each_input, each_output)] = dist
            if path[1] not in G.optimised_voisin[each_input]:
                G.optimised_voisin[each_input].append(path[1])

print(G.optimised_distances, file=sys.stderr)
#print(G.edges, file=sys.stderr)

#G.show()

while True:
    start_time = time()
    
    S.new_turn()
    
    entity_count = int(input())  # the number of entities (e.g. factories and troops)
    for i in range(entity_count):
        entity_id, entity_type, arg_1, arg_2, arg_3, arg_4, arg_5 = input().split()
        entity_id = int(entity_id)
        arg_1 = int(arg_1)
        arg_2 = int(arg_2)
        arg_3 = int(arg_3)
        arg_4 = int(arg_4)
        arg_5 = int(arg_5)
        if entity_type == "FACTORY":
            if not entity_id in Factory.instances.keys():
                Factory(entity_id, arg_1, arg_2, arg_3)
            else :
                Factory.instances[entity_id].update(arg_1, arg_2, arg_3)
        elif entity_type == "TROOP":
            #print(entity_id, entity_type, arg_1, arg_2, arg_3, arg_4, arg_5, file=sys.stderr)
            Cyborg(entity_id, arg_1, arg_2, arg_3, arg_4, arg_5)
        elif entity_type == "BOMB":
            Bomb(entity_id, arg_1, arg_2, arg_3)
    
    #print(S.scores, S.productivity ,file=sys.stderr)  
    
    S.clear_risk_of_impact() 
    S.set_availability()
    
    #print("",file=sys.stderr)
    #for each in Cyborg.my_robots.values():
    #    each.show()
    #print("",file=sys.stderr)
    #for each in Cyborg.ennemies.values():
    #    each.show()
    #print("",file=sys.stderr)
    #for each in Factory.my_bases.values():
    #    each.show()
    #print("",file=sys.stderr)
    #for each in Factory.instances.values():
    #    each.show()
    #print("",file=sys.stderr)
    #for each in Bomb.instances.values():
    #    each.show()
    S.create_strategy()
    S.command.append("MSG tour: %s - time: %.4s ms" %(S.loop, (time()-start_time)*1000))
    print(";".join(S.command))
    print("WAIT")
    S.loop += 2 