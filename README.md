# Thanks
Cornerstone is inspired by many other crates out there, here's a few worth mentioning:
+ [BehaviorTree.CPP](https://github.com/BehaviorTree/BehaviorTree.CPP)
+ [rusty-behavior-tree-lite](https://github.com/msakuta/rusty-behavior-tree-lite)
+ [bonsai](https://github.com/Sollimann/bonsai)

# Supported Features
- Composite Node
  - [x] Sequence
  - [x] Parallel
  - [x] Selector
  - [ ] Switch
- Decorator Node
  - [x] ForceSuccess
  - [x] ForceFailure
  - [x] Invert
  - [x] Repeat
  - [x] Subtree
  - [ ] Timeout
  - [x] Retry
- Action Node
  - [x] SetBlackboard
- Xml Parsing 
  - [x] Subtree
  - [x] Ports remapping
- Observe
  - [ ] record/replay
  - [ ] visualize
  - [ ] ~~user pause/resume~~