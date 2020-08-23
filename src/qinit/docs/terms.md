# Terms

A list of terms I use in the comments here

- Sphere: A thing that can be started - a stage, a task
- Sphere Template: An abstract definition which can be used to create Spheres
- Start Condition: A condition on a task or stage that moves the stage from "starting" to "started"
e.g. if a service is known to create a Unix socket, it can have a Start Condition to not be considered "started"
until that socket is opened
- Hard Sphere: A (name, args) pair where args contains all of the required arguments for the sphere template with the given name
- Soft Sphere: A (name, args) pair where args _does not_ contain all of the required arguments for the sphere template with the given name