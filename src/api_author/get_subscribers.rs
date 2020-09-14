#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]

use iota_streams::app_channels::{
    api::tangle::{Address, Author, Transport}
    , message
};
use failure::{Fallible, ensure, bail};


pub fn get_subscriptions_and_share_keyload<T: Transport>(author: &mut Author, channel_address: &String, subscribe_message_identifier: &String, client: &mut T, send_opt: T::SendOptions, recv_opt: T::RecvOptions) -> Fallible<Address> where T::SendOptions: Copy {
    
    println!("Receiving Subscribe messages");

    // Use the IOTA client to find transactions with the corresponding channel address and tag

    let subscription_link = match Address::from_str(&channel_address, &subscribe_message_identifier) {
        Ok(subscription_link) => subscription_link,
        Err(()) => bail!("Failed to create Address from {}:{}", &channel_address, &subscribe_message_identifier),
    };
    let subscribers = client.recv_messages_with_options(&subscription_link, recv_opt).unwrap();
    
    // Iterate through all the transactions
    let mut found_valid_msg = false;
    for tx in subscribers.iter() {
        let header = tx.parse_header().unwrap();
        ensure!(header.check_content_type(&message::SUBSCRIBE));
        // Process the message and read the subscribers' keys
        author.unwrap_subscribe(header.clone()).unwrap();
        println!("Found and verified {} message", header.content_type());
        found_valid_msg = true;
    }

    // Make sure that at least one of the messages were valid 
    ensure!(found_valid_msg);
    println!("Sending keyload");

    // Publish a Keyload message for all the subscribers whose `Subscribe` messages have been processed
    let keyload = author.share_keyload_for_everyone(&subscription_link).unwrap();
    // Convert the message to a bundle and send it to a node
    client.send_message_with_options(&keyload.0, send_opt).unwrap();
    client.send_message_with_options(&keyload.1.clone().unwrap(), send_opt).unwrap();
    println!("Keyload message at {}", &keyload.0.link.msgid);
    println!("Sequenced message at {}", &keyload.1.clone().unwrap().link.msgid);
    
    println!("Published keyload message");
    Ok(keyload.1.clone().unwrap().link)
}