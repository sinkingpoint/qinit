extern crate breuer;

use breuer::{
    FreudianBus, FreudianError, FreudianProduceMessageRequest, FreudianResponse, FreudianSubscriptionRequest, FreudianTopicRequest,
};

#[test]
fn test_bus_adds_topic() {
    let mut bus = FreudianBus::new();

    // First add should pass
    assert!(bus.create_topic(FreudianTopicRequest::new("test".to_owned())).is_ok());

    // Second add should fail, as the topic already exists
    let second_result = bus.create_topic(FreudianTopicRequest::new("test".to_owned()));
    assert!(second_result.is_err());
    assert_eq!(second_result.unwrap_err(), FreudianError::TopicAlreadyExists);
}

#[test]
fn test_bus_removes_topic() {
    let mut bus = FreudianBus::new();

    // Trying to remove with no topics should fail as the topic doesn't exist
    let try_remove = bus.delete_topic(FreudianTopicRequest::new("test".to_owned()));
    assert!(try_remove.is_err());
    assert_eq!(try_remove.unwrap_err(), FreudianError::TopicDoesntExist);

    // First add should pass
    assert!(bus.create_topic(FreudianTopicRequest::new("test".to_owned())).is_ok());

    // Remove should pass
    assert!(bus.delete_topic(FreudianTopicRequest::new("test".to_owned())).is_ok());

    // Second add should pass, as we deleted the first
    assert!(bus.create_topic(FreudianTopicRequest::new("test".to_owned())).is_ok());
}

#[test]
fn test_bus_subscribes() {
    let mut bus = FreudianBus::new();

    // Subscribe with the topic not existing should error
    let try_remove = bus.subscribe(FreudianTopicRequest::new("test".to_owned()));
    assert!(try_remove.is_err());
    assert_eq!(try_remove.unwrap_err(), FreudianError::TopicDoesntExist);

    // First add should pass
    assert!(bus.create_topic(FreudianTopicRequest::new("test".to_owned())).is_ok());

    // Should be able to subscribe twice
    let sub1 = bus.subscribe(FreudianTopicRequest::new("test".to_owned()));
    assert!(sub1.is_ok());

    let sub2 = bus.subscribe(FreudianTopicRequest::new("test".to_owned()));
    assert!(sub2.is_ok());
}

#[test]
fn test_bus_unsubscribes() {
    let mut bus = FreudianBus::new();

    let try_unsub = bus.unsubscribe(FreudianSubscriptionRequest::new([0; 16]));
    assert!(try_unsub.is_err());
    assert_eq!(try_unsub.unwrap_err(), FreudianError::SubscriptionDoesntExist);

    // First add should pass
    assert!(bus.create_topic(FreudianTopicRequest::new("test".to_owned())).is_ok());

    if let Ok(FreudianResponse::Subscription(uuid)) = bus.subscribe(FreudianTopicRequest::new("test".to_owned())) {
        let uuid: FreudianSubscriptionRequest = uuid.into();
        assert!(bus.unsubscribe(uuid.clone()).is_ok());

        // We can't unsub twice
        assert!(bus.unsubscribe(uuid.clone()).is_err());
    } else {
        assert!(false, "Invalid response from subscription");
    }
}

#[test]
fn test_bus_message_pipeline() {
    let mut bus = FreudianBus::new();

    // Try produce a message into the topic, which should bail because the topic doesn't exist
    let try_produce = bus.produce_message(FreudianProduceMessageRequest::new("test".to_owned(), vec![0; 16]));
    assert!(try_produce.is_err());
    assert_eq!(try_produce.unwrap_err(), FreudianError::TopicDoesntExist);

    // Make the topic
    assert!(bus.create_topic(FreudianTopicRequest::new("test".to_owned())).is_ok());

    // Producing a second time should bail because no one is listening yet
    let try_produce = bus.produce_message(FreudianProduceMessageRequest::new("test".to_owned(), vec![0; 16]));
    assert!(try_produce.is_err());
    assert_eq!(try_produce.unwrap_err(), FreudianError::NoSubscribers);

    // We should be able to subscribe
    if let Ok(FreudianResponse::Subscription(uuid)) = bus.subscribe(FreudianTopicRequest::new("test".to_owned())) {
        if let Ok(FreudianResponse::Subscription(second_uuid)) = bus.subscribe(FreudianTopicRequest::new("test".to_owned())) {
            // And Producing should now be successful messages
            assert!(bus
                .produce_message(FreudianProduceMessageRequest::new("test".to_owned(), vec![0; 16]))
                .is_ok());
            assert!(bus
                .produce_message(FreudianProduceMessageRequest::new("test".to_owned(), vec![1; 16]))
                .is_ok());
            assert!(bus
                .produce_message(FreudianProduceMessageRequest::new("test".to_owned(), vec![2; 16]))
                .is_ok());

            // We should be able to get the three messages back, in three seperate calls
            let sub_request: FreudianSubscriptionRequest = uuid.into();
            let msg = bus.consume_message(sub_request.clone());
            assert!(msg.is_ok());
            assert_eq!(msg.unwrap(), FreudianResponse::Message(vec![0; 16]));

            let msg = bus.consume_message(sub_request.clone());
            assert!(msg.is_ok());
            assert_eq!(msg.unwrap(), FreudianResponse::Message(vec![1; 16]));

            let msg = bus.consume_message(sub_request.clone());
            assert!(msg.is_ok());
            assert_eq!(msg.unwrap(), FreudianResponse::Message(vec![2; 16]));

            // And a fourth message should fail
            assert!(bus.consume_message(sub_request.clone()).is_err());

            // Fetching with the second UUID should allow us to read all the messages again
            let sub_request2: FreudianSubscriptionRequest = second_uuid.into();
            let msg = bus.consume_message(sub_request2.clone());
            assert!(msg.is_ok());
            assert_eq!(msg.unwrap(), FreudianResponse::Message(vec![0; 16]));

            let msg = bus.consume_message(sub_request2.clone());
            assert!(msg.is_ok());
            assert_eq!(msg.unwrap(), FreudianResponse::Message(vec![1; 16]));

            let msg = bus.consume_message(sub_request2.clone());
            assert!(msg.is_ok());
            assert_eq!(msg.unwrap(), FreudianResponse::Message(vec![2; 16]));

            // And a fourth message should fail
            assert!(bus.consume_message(sub_request2.clone()).is_err());

            // Subbing a third time, the subscription should start at the head of the messages, so reading should fail
            if let Ok(FreudianResponse::Subscription(third_uuid)) = bus.subscribe(FreudianTopicRequest::new("test".to_owned())) {
                let sub_request3: FreudianSubscriptionRequest = third_uuid.into();
                assert!(bus.consume_message(sub_request3.clone()).is_err());
                assert!(bus
                    .produce_message(FreudianProduceMessageRequest::new("test".to_owned(), vec![3; 16]))
                    .is_ok());

                assert_eq!(
                    bus.consume_message(sub_request.clone()).unwrap(),
                    FreudianResponse::Message(vec![3; 16])
                );
                assert_eq!(
                    bus.consume_message(sub_request2.clone()).unwrap(),
                    FreudianResponse::Message(vec![3; 16])
                );
                assert_eq!(
                    bus.consume_message(sub_request3.clone()).unwrap(),
                    FreudianResponse::Message(vec![3; 16])
                );
            } else {
                assert!(false, "Invalid response from subscription");
            }
        } else {
            assert!(false, "Invalid response from subscription");
        }
    } else {
        assert!(false, "Invalid response from subscription");
    }
}
