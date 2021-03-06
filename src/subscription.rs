use frame::Frame;
use subscription::AckOrNack::{Ack, Nack};
use header::HeaderList;
use std::sync::mpsc::Sender;

#[derive(Copy,Clone)]
pub enum AckMode {
  Auto,
  Client,
  ClientIndividual
}

impl AckMode {
  pub fn as_text(&self) -> &'static str {
    match *self {
      AckMode::Auto => "auto",
      AckMode::Client => "client",
      AckMode::ClientIndividual => "client-individual"
    }
  }
}

#[derive(Clone, Copy)]
pub enum AckOrNack {
  Ack,
  Nack
}

pub trait MessageHandler {
  fn on_message(&mut self, &Frame) -> AckOrNack;
}

pub struct Subscription <'a> { 
  pub id : String,
  pub destination: String,
  pub ack_mode: AckMode,
  pub headers: HeaderList,
  pub handler: Box<MessageHandler + 'a>
}

impl <'a> Subscription <'a> {
  pub fn new(id: u32, destination: &str, ack_mode: AckMode, headers: HeaderList, message_handler: Box<MessageHandler + 'a>) -> Subscription <'a> {
    Subscription {
      id: format!("stomp-rs/{}",id),
      destination: destination.to_string(),
      ack_mode: ack_mode,
      headers: headers,
      handler: message_handler
    }
  }
}

pub trait ToMessageHandler <'a> {
  fn to_message_handler(self) -> Box<MessageHandler + 'a>;
}

impl <'a, T: 'a> ToMessageHandler <'a> for T where T: MessageHandler {
  fn to_message_handler(self) -> Box<MessageHandler + 'a> {
    Box::new(self) as Box<MessageHandler>
  }
} 

impl <'a> ToMessageHandler <'a> for Box<MessageHandler + 'a> {
  fn to_message_handler(self) -> Box<MessageHandler + 'a> {
    self 
  }
} 
// Support for Sender<T> in subscriptions

struct SenderMessageHandler {
  sender: Sender<Frame>
}

impl MessageHandler for SenderMessageHandler {
  fn on_message(&mut self, frame: &Frame) -> AckOrNack {
    debug!("Sending frame...");
    match self.sender.send(frame.clone()) {
      Ok(_) => Ack,
      Err(error) => {
        error!("Failed to send frame: {}", error);
        Nack
      }
    }
  }
}

impl <'a> ToMessageHandler<'a> for Sender<Frame> {
  fn to_message_handler(self) -> Box<MessageHandler + 'a> {
    Box::new(SenderMessageHandler{sender : self}) as Box<MessageHandler>
  }
}

impl <F> MessageHandler for F where F : FnMut(&Frame) -> AckOrNack {
  fn on_message(&mut self, frame: &Frame) -> AckOrNack {
    debug!("Passing frame to closure...");
    self(frame)
  }
}
