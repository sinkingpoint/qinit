@0xd80cff29a95f9c6b;

# RestartMode represents how the task will be restarted when stopped
enum RestartMode {
    always @0; # Always restart the task (Should be used for things that should never be stopped, even by a user)
    unlessStopped @1; # Restart the task on an error, or when it exits cleanly, as long as it wasn't manually stopped
    onError @2; # Restart the task if it crashes
    never @3; # Never restart the task
}

struct Map(Key, Value) {
  entries @0 :List(Entry);
  struct Entry {
    key @0 :Key;
    value @1 :Value;
  }
}

# Represents a dependancy of a Task on another Task
struct DependencyDef {
    name @0 :Text; # The name of the task that we have a dependancy on
    args @1 :Map(Text, Text); # The arguments of the task we have a dependancy on
}

struct UnixSocket {
    path @0 :Text; # The path 
}

struct Task {
    name @0 :Text;
    description @1 :Text;
    user :union {
        id @2 :UInt32;
        name @3 :Text;
    }
    group :union {
        id @4 :UInt32;
        name @5 :Text;
    }
    args @6 :List(Text);
    command @7 :Text;
    restartMode @8 :RestartMode;
    requirements @9 :List(DependencyDef);
    unixSockets @10 :List(UnixSocket);
}

struct Stage {
    requirements @0 :List(DependencyDef);
}
