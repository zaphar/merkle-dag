---- MODULE scratch ----
EXTENDS TLC, Integers
LOCAL INSTANCE Naturals
LOCAL INSTANCE Sequences

Boolean == {TRUE, FALSE}
Node == 1..10
GraphType == [Node \X Node -> Boolean]
Eval == GraphType
====