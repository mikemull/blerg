# blerg
Toy Rust program to mess with network stuff

Right now it grabs every Layer 2 packet on a specified interface for some number of packets, and counts 'em up.
This apparently useless function is kind of a fun way to discover stuff on your network.  You run it like:

```
   blerg <interface> <num_packets>
```

It expects to find a file called _macs.csv_ in the same directory, where each line has mac_address, name.
