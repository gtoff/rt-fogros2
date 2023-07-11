# ROS Launchable FogROS2-SGC

### Configuartion file 

FogROS2-SGC requires a separate configuration file to expose the topics. While it supports automatic topic scanning, we want the users to expose the topics globally only if they want to, to enhance the isolation and protect the privacy of the robots and services. An example of the configuration file can be found as following: 

```
identifiers: 
  # crypto for the task (same for all robots/services on the same task)
  task: test_cert 
  # crypto unique for the robot 
  # note #0 : 
  # for now, it's not bound to an actual certificate yet, name whatever you want
  # but make sure to match this with the assignment 
  # note #1 : this value is overriden by rosparam's whoami
  # note #2 : feel free to define it here or define it at rosparam
  whoami: machine_talker 
  
topics:
  - topic_name: /chatter
    topic_type: std_msgs/msg/String

# declare possible states 
# pub: publish to the swarm
# sub: subscribe to the swarm
# note this is reversed from prior version of SGC config file
state_machine: 
  talker: 
    topics:
      - /chatter: pub 
  listener: 
    topics:
      - /chatter: sub 

# name: state
# name need to match the identitifer's whoami
# state should be declared in possible states 
# Phase 1: only allow changing the assignment at runtime 
assignment:
  machine_talker: talker
  machine_listener: listener
```

We observe that the cloud services are usually the reversed of the robots' configuration file (e.g. cloud is the publisher then robot is the subscriber). We want the user only need to write and distribute one copy of the configuration file, and mark themselves as different only at `whoami` in `identifiers`. This value is overriden by the ros2 launch file's parameter. 

In the example, the configuration configures `whoami` as `machine_talker`, which is assigned with the `talker`'s state (at a`assignment`). The `talker` state is defined to `pub` (publish) to the topic `chatter`. The `chatter` is configured to be of type `std_msgs/msgs/String`. Vise versa, the listener only need to set `whoami` as `machine_listener`, and the states (how the topics are handled) are automated. 