---- MODULE dag ----
EXTENDS TLC, Integers
CONSTANT
    NumNodes, Null

Nodes == 1..NumNodes

VARIABLE graph, pc, tmp, edges

vars == << graph, pc, tmp, edges >>

Init == /\ graph = {}
        /\ pc = "GetEdge"
        /\ tmp = Null
        /\ edges = {x \in (Nodes \X Nodes): x[1] # x[2]}

GetEdge == /\ tmp = Null
           /\ \/ /\ graph = {}
                 /\  tmp' = Null
              \/ tmp' = CHOOSE e \in edges: (e \notin graph /\ e[1] # e[2])
           /\ edges' = edges \ {tmp'}
           /\ pc' = "AddEdge"
           /\ UNCHANGED graph

AddEdge == /\ tmp # Null
           /\ graph' = (graph \union {tmp})
           /\ pc' = "Done"
           /\ tmp' = Null
           /\ UNCHANGED edges

Terminating == /\ pc = "Done"
               /\ UNCHANGED vars

Next == \/ GetEdge \/ AddEdge
        \/ Terminating

Termination == <>(pc = "Done")

Spec == Init /\ [][Next]_vars
====
