# Freud

This is a service similar to dbus to handle arbitrary messaging between services, based on the Kafka queing system. The main difference is that
we don't care about replayability or resiliency. To that end, we make some design assumptions:

- Messages are assumed to only be destined to the local system and only relevant to the current state of the system and thus are 
  invalid if the system reboots or crashes (Messages are not persisted on disk)
- Messages are assumed to only be relevant to existing subscribers. If a message is entered into a topic with no subscribers, 
  or after all subscribers of a topic have read the message, the message is deleted

## Definitions

### Topic
A topic is a collection of messages that can be subscribed to. Messages in a topic are stored in the order they are produced and consumed in that same order.
Topics can be arbitrarily created by running services for the sake of communicating to other services, and optionally have an ACL to determine which of these
services can read/write to them.

### ACL
An ACL is a whitelist/blacklist of rules defining which users, groups, and services are/aren't allowed to produce, or subscribe to a topic

### Message
A message is an arbitrary string of bytes, marked by an ID number (Determined by its place in the topic), and a nanosecond precision timestamp

### Producer
A producer is a service that produces messages into a given topic

### Subscriber
A subscriber is a service that subscribes to messages in a given topic and implies that is has some interest in performing actions based on those messages

### Requesting Service
A requesting service is a service that sends requests a to Freud service, and expects some response

## Components

### Freudian

Freudian is the message broker daemon that handles message transport between producers and topics, and topics and subscribers. It receives messages in the format:

```
+-----------------------------+-------------------------------------------------------+------------------------------------------------------------------------------+
|                             |                                                       |                                                                              |
|       Message Type          |                    Message Length                     |                                    Message                                   |
|         (4 Bytes)           |                      (4 Bytes)                        |                           (`Message Length` Bytes)                           |
|                             |                                                       |                                                                              |
+-----------------------------+-------------------------------------------------------+------------------------------------------------------------------------------+
```

where the format of the message is defined by message type. Requests to Freudian return a 8 bit status code indicating the result of the request, along with a
similarly arbitrary response payload:

```
+-----------------------------+-------------------------------------------------------+------------------------------------------------------------------------------+
|                             |                                                       |                                                                              |
|       Response Code         |                    Message Length                     |                                    Message                                   |
|         (4 Bytes)           |                      (4 Bytes)                        |                           (`Message Length` Bytes)                           |
|                             |                                                       |                                                                              |
+-----------------------------+-------------------------------------------------------+------------------------------------------------------------------------------+
```


#### Possible Message Types

##### `0` - Create Topic

```
+------------------------------------------+------------------------------------------------------------------------+
|                                          |                                                                        |
|             Topic Name Length            |                                Topic Name                              |
|                 (4 bytes)                |                                                                        |
|                                          |                                                                        |
+------------------------------------------+------------------------------------------------------------------------+
```

A create topic request creates a topic of a given name, if it doesn't already exist. 

- If the topic did not already exist, creates the topic and returns Ok
- If the topic does already exist, but is marked for deletion, marks the topic for recreation, and returns `Ok`. Messages send to topics marked for recreation are buffered until all subscribers have read the deletion message
- If the topic already exists, and is _not_ marked for deletion, returns `Nothing Happened`. Note: this _does not_ imply that the service has permissions to write to the already existing topic
- If the requesting service does not have the required permissions to create a new topic, returns `No`

##### `1` - Delete Topic

```
+------------------------------------------+------------------------------------------------------------------------+
|                                          |                                                                        |
|             Topic Name Length            |                                Topic Name                              |
|                 (4 bytes)                |                                                                        |
|                                          |                                                                        |
+------------------------------------------+------------------------------------------------------------------------+
```

A delete topic requests marks a topic for deletion. It pushes a single message into the topic indicating to the subscribers of the impending topic deletion
and once this message is read by all subscribers, the topic is deleted.

- If the topic was successfully marked for deletion, returns `Ok`. Note that this does not mean that the topic is actually deleted - it can stay around for up to `SUBSCRIBER_TIMEOUT_SECS` seconds waiting for subscribers to acknowledge the topic deletion before being deleted.
- If the topic doesn't exist, returns `Nothing Happened`
- If the requesting service is _not_ the service that created the topic, and has not been given permission to delete the topic, returns `No`

##### `2` - (Like and )Subscribe

```
+------------------------------------------+------------------------------------------------------------------------+
|                                          |                                                                        |
|             Topic Name Length            |                                Topic Name                              |
|                 (4 bytes)                |                                                                        |
|                                          |                                                                        |
+------------------------------------------+------------------------------------------------------------------------+
```

A subscribe request indicates the requesting processes desire to receive messages on a given topic.

- If the requesting process is allowed to subscribe to the topic, and the registration of its subscription was successful, returns `Ok`. The response payload will
contain a 16 byte GUID denoted the `subscriber id` which must be used in subsequent "Get Message" requests. Note that Freud does not distinguish between subscribers
except through this ID thus if the same process subscribes twice to the same topic, it will be issued two subscriber IDs - we will *not* return `Nothing Happened`.
- If the requesting process is _not_ allowed to subscribe to the given topic, returns an empty `No` response

##### `3` - Unsubscribe

```
+------------------------------------------+
|                                          |
|             Subscriber ID                |
|              (16 bytes)                  |
|                                          |
+------------------------------------------+
```

An unsubscribe request deletes the subscription indicated by the Subscriber ID. The same ID can no longer be used to get messages from the given topic.

- If the subscriber ID is valid, deletes subscription and returns `Ok`
- If the subscriber ID is _invalid_, returns `Doesn't Exist`

##### `4` - Produce Message

```
+------------------------------------------+-------------------------------------+------------------------------------------------------------------------+
|                                          |                                     |                                                                        |
|             Topic Name Length            |             Topic Name              |                                Message                                 |
|                 (4 bytes)                |                                     |                                                                        |
|                                          |                                     |                                                                        |
+------------------------------------------+-------------------------------------+------------------------------------------------------------------------+
```

Produce Message takes the `Message` of the request and enqueues it into the given topic, if it exists.

- If the topic exists, and the and the requesting service is allowed to produce messages into it, but there are no subscribers to the topic, the message
will be dropped and an empty `Nothing Happened` response will be returned
- If the topic exists, and the requesting service is allowed to produce messages into it, returns `Ok`, and a `message id` - an integer representing a monotonically
increasing ID of messages in the topic
- If the topic doesn't exist, returns `Doesn't Exist`
- If the topic does exist, but the requesting service does not have permissions to put messages in it, returns `No`

##### `5` - Get Message

```
+------------------------------------------+------------------------------------------+
|                                          |                                          |
|             Subscriber ID                |             Max Blocking Secs            |
|              (16 bytes)                  |                 (4 Bytes)                |
|                                          |                                          |
+------------------------------------------+------------------------------------------+
```

Get Message takes a subscriber ID (Previous received from a `Subscribe` request), and blocks for up to `Max Blocking Secs` seconds 
(or forever, if Max Blocking Secs is < 0) until there is a message available to be consumed. Once there is,
the subscribers `Offset` is updated to indicate that the message has been consumed by the subscriber and the message is returned.

- If a message is available, returns `Ok` and the message as a payload
- If `Max Blocking Secs` is reached with no message available, returns `Nothing Happened`
- If the given subscriber ID doesn't exist, returns `Doesn't Exist`

#### Status codes

##### `0` - Ok

Indicates that something happened, and that something was successful in completing the requested action

##### `1` - Nothing Happened

Indicates that the request was successful, but we didn't have to do anything to fulfil it

##### `2` - No

Indicates that the requesting service does not have permissions to perform the given request

##### `3` Doesn't Exist

Indicates that a requesting service is attempting to interact with a resource (Generally a topic) that doesn't exist

##### `4` Malformed Request

Indicates that the request was in a format inappropriate for the message type

##### `5` Server Error

Indicates that an internal server error occurred