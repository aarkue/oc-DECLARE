# OC-DECLARE

Python bindings for the OC-DECLARE implementation.
OC-DECLARE allows discovering declarative synchronized object-centric process constraints from Object-Centric Event Logs (OCEL) in the OCEL 2.0 format.

More details can also be found in the original paper [OC-DECLARE: Discovering Object-Centric Declarative Patterns with Synchronization](https://doi.org/10.1007/978-3-032-02867-9_11), the [OC-DECLARE GitHub Repo](https://github.com/aarkue/OC-DECLARE), or the [Rust4PM Github Repo](https://github.com/aarkue/rust4pm).

If you have any questions or run into any problems, please open an issue [here](https://github.com/aarkue/oc-DECLARE/issues/new).

**See below for a usage example. The Jupyter Notebook version of the example is available [on GitHub](https://github.com/aarkue/oc-DECLARE/blob/main/oc_declare_python/example.ipynb).**



---




```python
import oc_declare
```


```python
arc = oc_declare.OCDeclareArc("Load Truck","Depart","EF",1,None,all_ots=["items"],each_ots=["orders"],any_ots=[])
arc
```




    OC-DECLARE Arc: EF(Load Truck, Depart, Each(orders), All(items),1,∞)




```python
arc.any_ots = ["employees"]
arc
```




    OC-DECLARE Arc: EF(Load Truck, Depart, Each(orders), All(items), Any(employees),1,∞)




```python
ocel = oc_declare.import_ocel2("../../../../dow/ocel/ContainerLogistics.json")
```


```python
res = oc_declare.discover(ocel,0.2,acts_to_use=["Load Truck", "Pick Up Empty Container","Depart"],o2o_mode="None")
```


```python
for arc in res:
    print(arc.to_string())
    print(oc_declare.check_conformance(ocel,arc))
    print("---")
```

    EP(Depart, Pick Up Empty Container, Each(Container),1,∞)
    1.0
    ---
    EF(Pick Up Empty Container, Depart, Any(Container),1,∞)
    0.978978978978979
    ---
    EF(Load Truck, Depart, Any(Container),1,∞)
    0.9793697359704742
    ---
    DF(Pick Up Empty Container, Load Truck, Any(Container),1,∞)
    0.997997997997998
    ---
    EP(Load Truck, Pick Up Empty Container, Any(Container),1,∞)
    0.9986751206586543
    ---
    EP(Depart, Load Truck, Each(Container),1,∞)
    1.0
    ---



```python
all_res = oc_declare.discover(ocel,0.2)
print(f"Discovered {len(all_res)} constraints!")

all_res
```

    Discovered 99 constraints!





    [OC-DECLARE Arc: EP(Drive to Terminal, Order Empty Containers, Any(Container),1,∞),
     OC-DECLARE Arc: EP(Bring to Loading Bay, Load Truck, Any(Container),1,∞),
     OC-DECLARE Arc: DF(Pick Up Empty Container, Load Truck, Any(Container),1,∞),
     OC-DECLARE Arc: EP(Depart, Bring to Loading Bay, Each(Container),1,∞),
     OC-DECLARE Arc: EP(Depart, Weigh, Each(Container),1,∞),
     OC-DECLARE Arc: DF(Create Transport Document, Book Vehicles, Any(Transport Document),1,∞),
     OC-DECLARE Arc: EP(Create Transport Document, Register Customer Order, Any(Customer Order),1,∞),
     OC-DECLARE Arc: EF(Load Truck, Bring to Loading Bay, Any(Container),1,∞),
     OC-DECLARE Arc: DP(Weigh, Drive to Terminal, Any(Container),1,∞),
     OC-DECLARE Arc: EF(Weigh, Load to Vehicle, Any(Container),1,∞),
     OC-DECLARE Arc: DP(Order Empty Containers, Book Vehicles, Any(Transport Document),1,∞),
     OC-DECLARE Arc: EF(Order Empty Containers, Load to Vehicle, Each(Container),1,∞),
     OC-DECLARE Arc: EP(Place in Stock, Pick Up Empty Container, Any(Container),1,∞),
     OC-DECLARE Arc: EP(Bring to Loading Bay, Weigh, Any(Container),1,∞),
     OC-DECLARE Arc: EP(Weigh, Pick Up Empty Container, Any(Container),1,∞),
     OC-DECLARE Arc: DF(Register Customer Order, Create Transport Document, Any(Customer Order),1,∞),
     OC-DECLARE Arc: EP(Place in Stock, Order Empty Containers, Any(Container),1,∞),
     OC-DECLARE Arc: EF(Load Truck, Drive to Terminal, Any(Container,Truck),1,∞),
     OC-DECLARE Arc: EF(Bring to Loading Bay, Depart, Any(Container),1,∞),
     OC-DECLARE Arc: EF(Drive to Terminal, Load to Vehicle, Any(Container),1,∞),
     OC-DECLARE Arc: EP(Depart, Load Truck, Each(Container),1,∞),
     OC-DECLARE Arc: DF(Load to Vehicle, Depart, Any(Container,Vehicle),1,∞),
     OC-DECLARE Arc: DF(Drive to Terminal, Weigh, Any(Container),1,∞),
     OC-DECLARE Arc: EF(Pick Up Empty Container, Depart, Any(Container),1,∞),
     OC-DECLARE Arc: EP(Depart, Pick Up Empty Container, Each(Container),1,∞),
     OC-DECLARE Arc: EF(Load Truck, Depart, Any(Container),1,∞),
     OC-DECLARE Arc: EP(Load to Vehicle, Book Vehicles, Any(Vehicle),1,∞),
     OC-DECLARE Arc: EP(Load to Vehicle, Order Empty Containers, Any(Container),1,∞),
     OC-DECLARE Arc: EF(Order Empty Containers, Load Truck, Each(Container),1,∞),
     OC-DECLARE Arc: DP(Place in Stock, Weigh, Any(Container,Forklift),1,∞),
     OC-DECLARE Arc: EP(Drive to Terminal, Pick Up Empty Container, Any(Container),1,∞),
     OC-DECLARE Arc: DF(Bring to Loading Bay, Load to Vehicle, Any(Container,Forklift),1,∞),
     OC-DECLARE Arc: EP(Pick Up Empty Container, Order Empty Containers, Any(Container),1,∞),
     OC-DECLARE Arc: DF(Book Vehicles, Depart, Each(Vehicle), Any(Transport Document),1,∞),
     OC-DECLARE Arc: EP(Load Truck, Collect Goods, Any(Handling Unit),1,∞),
     OC-DECLARE Arc: DF(Weigh, Place in Stock, Any(Container,Forklift),1,∞),
     OC-DECLARE Arc: EP(Load to Vehicle, Place in Stock, Any(Container),1,∞),
     OC-DECLARE Arc: EF(Order Empty Containers, Bring to Loading Bay, Each(Container),1,∞),
     OC-DECLARE Arc: EF(Order Empty Containers, Weigh, Each(Container),1,∞),
     OC-DECLARE Arc: EF(Order Empty Containers, Place in Stock, Any(Container),1,∞),
     OC-DECLARE Arc: EF(Place in Stock, Depart, Any(Container),1,∞),
     OC-DECLARE Arc: EF(Place in Stock, Load to Vehicle, Any(Container),1,∞),
     OC-DECLARE Arc: DF(Reschedule Container, Depart, Any(Container,Transport Document,Vehicle),1,∞),
     OC-DECLARE Arc: EP(Reschedule Container, Weigh, Any(Container),1,∞),
     OC-DECLARE Arc: DP(Reschedule Container, Order Empty Containers, Any(Container,Transport Document),1,∞),
     OC-DECLARE Arc: DP(Bring to Loading Bay, Place in Stock, Any(Container),1,∞),
     OC-DECLARE Arc: EF(Load Truck, Weigh, Any(Container),1,∞),
     OC-DECLARE Arc: EP(Load to Vehicle, Load Truck, Any(Container),1,∞),
     OC-DECLARE Arc: DF(Order Empty Containers, Depart, Each(Container), Any(Transport Document),1,∞),
     OC-DECLARE Arc: EP(Reschedule Container, Drive to Terminal, Any(Container),1,∞),
     OC-DECLARE Arc: EF(Drive to Terminal, Place in Stock, Any(Container),1,∞),
     OC-DECLARE Arc: EF(Pick Up Empty Container, Bring to Loading Bay, Any(Container),1,∞),
     OC-DECLARE Arc: DF(Book Vehicles, Order Empty Containers, Any(Transport Document),1,∞),
     OC-DECLARE Arc: EP(Depart, Create Transport Document, Each(Transport Document),1,∞),
     OC-DECLARE Arc: EF(Create Transport Document, Order Empty Containers, Any(Transport Document),1,∞),
     OC-DECLARE Arc: EP(Load Truck, Pick Up Empty Container, Any(Container),1,∞),
     OC-DECLARE Arc: DP(Load to Vehicle, Bring to Loading Bay, Any(Container,Forklift),1,∞),
     OC-DECLARE Arc: EF(Drive to Terminal, Depart, Any(Container),1,∞),
     OC-DECLARE Arc: EP(Bring to Loading Bay, Pick Up Empty Container, Any(Container),1,∞),
     OC-DECLARE Arc: EF(Pick Up Empty Container, Weigh, Any(Container),1,∞),
     OC-DECLARE Arc: EP(Book Vehicles, Create Transport Document, Any(Transport Document),1,∞),
     OC-DECLARE Arc: DP(Depart, Book Vehicles, Each(Transport Document), Any(Vehicle),1,∞),
     OC-DECLARE Arc: DP(Depart, Order Empty Containers, Each(Transport Document), Any(Container),1,∞),
     OC-DECLARE Arc: DP(Depart, Order Empty Containers, Each(Container), Any(Transport Document),1,∞),
     OC-DECLARE Arc: EP(Weigh, Load Truck, Any(Container),1,∞),
     OC-DECLARE Arc: EP(Load to Vehicle, Weigh, Any(Container),1,∞),
     OC-DECLARE Arc: EF(Order Empty Containers, Pick Up Empty Container, Each(Container),1,∞),
     OC-DECLARE Arc: DF(Place in Stock, Bring to Loading Bay, Any(Container),1,∞),
     OC-DECLARE Arc: DP(Reschedule Container, Bring to Loading Bay, Any(Container),1,∞),
     OC-DECLARE Arc: EP(Reschedule Container, Pick Up Empty Container, Any(Container),1,∞),
     OC-DECLARE Arc: EP(Reschedule Container, Book Vehicles, Any(Transport Document),1,∞),
     OC-DECLARE Arc: EP(Reschedule Container, Book Vehicles, Any(Vehicle),1,∞),
     OC-DECLARE Arc: EP(Reschedule Container, Create Transport Document, Any(Transport Document),1,∞),
     OC-DECLARE Arc: EP(Reschedule Container, Load Truck, Any(Container),1,∞),
     OC-DECLARE Arc: DF(Reschedule Container, Load to Vehicle, Any(Container,Vehicle),1,∞),
     OC-DECLARE Arc: DP(Drive to Terminal, Load Truck, Any(Container,Truck),1,∞),
     OC-DECLARE Arc: EF(Pick Up Empty Container, Load to Vehicle, Any(Container),1,∞),
     OC-DECLARE Arc: EP(Depart, Drive to Terminal, Each(Container),1,∞),
     OC-DECLARE Arc: DP(Depart, Load to Vehicle, Each(Container), Any(Vehicle),1,∞),
     OC-DECLARE Arc: EP(Load Truck, Order Empty Containers, Any(Container),1,∞),
     OC-DECLARE Arc: EF(Weigh, Bring to Loading Bay, Any(Container),1,∞),
     OC-DECLARE Arc: EF(Weigh, Depart, Any(Container),1,∞),
     OC-DECLARE Arc: EP(Weigh, Order Empty Containers, Any(Container),1,∞),
     OC-DECLARE Arc: EP(Load to Vehicle, Pick Up Empty Container, Any(Container),1,∞),
     OC-DECLARE Arc: EP(Order Empty Containers, Create Transport Document, Any(Transport Document),1,∞),
     OC-DECLARE Arc: DF(Collect Goods, Load Truck, Any(Handling Unit),1,∞),
     OC-DECLARE Arc: EF(Pick Up Empty Container, Drive to Terminal, Any(Container),1,∞),
     OC-DECLARE Arc: EF(Load Truck, Place in Stock, Any(Container),1,∞),
     OC-DECLARE Arc: EF(Drive to Terminal, Bring to Loading Bay, Any(Container),1,∞),
     OC-DECLARE Arc: EP(Bring to Loading Bay, Order Empty Containers, Any(Container),1,∞),
     OC-DECLARE Arc: EF(Pick Up Empty Container, Place in Stock, Any(Container),1,∞),
     OC-DECLARE Arc: EF(Create Transport Document, Depart, Any(Transport Document),1,∞),
     OC-DECLARE Arc: EF(Load Truck, Load to Vehicle, Any(Container),1,∞),
     OC-DECLARE Arc: EP(Load to Vehicle, Drive to Terminal, Any(Container),1,∞),
     OC-DECLARE Arc: EF(Order Empty Containers, Drive to Terminal, Each(Container),1,∞),
     OC-DECLARE Arc: EP(Place in Stock, Drive to Terminal, Any(Container),1,∞),
     OC-DECLARE Arc: EP(Bring to Loading Bay, Drive to Terminal, Any(Container),1,∞),
     OC-DECLARE Arc: EP(Depart, Place in Stock, Any(Container),1,∞),
     OC-DECLARE Arc: EP(Place in Stock, Load Truck, Any(Container),1,∞)]


