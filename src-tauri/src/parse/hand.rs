use chrono::{DateTime, FixedOffset};

// This structure will be used to compute the stats of the player
#[derive(Default, Debug, PartialEq)]
pub struct HandDetail {
  pub id: i64, // u32 is too small
  pub content: String,
  pub real_money: bool,
  pub date: DateTime<FixedOffset>,
  pub small_limit: f32,
  pub big_limit: f32,
  pub table_name: String,
  pub table_size: u8,
  pub button_position: u8, // usefull to shift position and guess real position
  pub players: [Option<Player>; 9],
  pub small_blind: Blind,
  pub big_blind: Blind,
  pub end: End, // NOTE: not used
  pub players_card: [Option<[String; 2]>; 9],
  pub preflop: Vec<Action>,
  pub flop: Vec<Action>,
  pub turn: Vec<Action>,
  pub river: Vec<Action>,
  pub flop_card: Option<[String; 3]>,
  pub turn_card: Option<String>,
  pub river_card: Option<String>,
}

use crate::db::models;
impl HandDetail {
  pub fn get_actions(&self) -> Vec<models::Action> {
    let mut actions: Vec<models::Action> = Vec::new();
    let mut sequence = 0;

    for action in &self.preflop {
      actions.push(HandDetail::create_action(
        action, "preflop", sequence, self.id,
      ));
      sequence += 1;
    }
    for action in &self.flop {
      actions.push(HandDetail::create_action(action, "flop", sequence, self.id));
      sequence += 1;
    }
    for action in &self.turn {
      actions.push(HandDetail::create_action(action, "turn", sequence, self.id));
      sequence += 1;
    }
    for action in &self.river {
      actions.push(HandDetail::create_action(
        action, "river", sequence, self.id,
      ));
      sequence += 1;
    }
    actions
  }

  fn create_action(action: &Action, moment: &str, sequence: i32, foreign: i64) -> models::Action {
    let mut object = models::Action {
      player: "".to_string(),
      hand: foreign,
      kind: "".to_string(),
      moment: moment.to_string(),
      sequence,
      amount1: 0., // placeholder
      amount2: 0., // placeholder
      allin: false,
    };
    match action {
      Action::Call(player, amount, allin) => {
        object.player = player.name.clone();
        object.amount1 = *amount;
        object.allin = *allin;
      }
      Action::Bet(player, amount, allin) => {
        object.player = player.name.clone();
        object.amount1 = *amount;
        object.allin = *allin;
      }
      Action::Raise(player, amount1, amount2, allin) => {
        object.player = player.name.clone();
        object.amount1 = *amount1;
        object.amount1 = *amount2;
        object.allin = *allin;
      }
      Action::Check(player) => {
        object.player = player.name.clone();
      }
      Action::Fold(player) => {
        object.player = player.name.clone();
      }
      Action::Leave(player) => {
        object.player = player.name.clone();
      }
      Action::UncalledBet(player, amount) => {
        object.player = player.name.clone();
        object.amount1 = *amount;
      }
    };
    object
  }

  pub fn get_hand(&self) -> models::Hand {
    models::Hand {
      id: self.id,
      content: self.content.clone(),
      real_money: self.real_money,
      time: self.date.timestamp(),
      table_name: self.table_name.clone(),
      table_size: self.table_size as i32,
      winner: self.end.winner.name.clone(),
      pot: self.end.pot,
      player1: self.players[0]
        .as_ref()
        .map_or(String::new(), |p| p.name.clone()),
      player2: self.players[1]
        .as_ref()
        .map_or(String::new(), |p| p.name.clone()),
      player3: self.players[2]
        .as_ref()
        .map_or(String::new(), |p| p.name.clone()),
      player4: self.players[3]
        .as_ref()
        .map_or(String::new(), |p| p.name.clone()),
      player5: self.players[4]
        .as_ref()
        .map_or(String::new(), |p| p.name.clone()),
      player6: self.players[5]
        .as_ref()
        .map_or(String::new(), |p| p.name.clone()),
      player7: self.players[6]
        .as_ref()
        .map_or(String::new(), |p| p.name.clone()),
      player8: self.players[7]
        .as_ref()
        .map_or(String::new(), |p| p.name.clone()),
      player9: self.players[8]
        .as_ref()
        .map_or(String::new(), |p| p.name.clone()),
      card1: self
        .flop_card
        .as_ref()
        .map_or(String::new(), |cards| cards[0].clone()),
      card2: self
        .flop_card
        .as_ref()
        .map_or(String::new(), |cards| cards[1].clone()),
      card3: self
        .flop_card
        .as_ref()
        .map_or(String::new(), |cards| cards[2].clone()),
      card4: self.turn_card.as_ref().map_or(String::new(), |c| c.clone()),
      card5: self
        .river_card
        .as_ref()
        .map_or(String::new(), |c| c.clone()),
    }
  }

  pub fn get_blinds(&self) -> (models::Blind, models::Blind) {
    // small and big
    let small = models::Blind {
      player: self.small_blind.player.name.clone(),
      hand: self.id,
      amount: self.small_blind.amount,
      kind: "small".to_string(),
    };
    let big = models::Blind {
      player: self.big_blind.player.name.clone(),
      hand: self.id,
      amount: self.big_blind.amount,
      kind: "big".to_string(),
    };
    (small, big)
  }

  pub fn get_hole_cards(&self) -> Vec<models::HoleCard> {
    let mut vec: Vec<models::HoleCard> = Vec::new();
    for i in 0..9 {
      if let Some(cards) = &self.players_card[i] {
        let player = {
          let opt = &self.players[i];
          match opt {
            Some(p) => p,
            None => panic!(),
          }
        };

        let hole = models::HoleCard {
          hand: self.id,
          player: player.name.to_string(),
          card1: cards[0].to_string(),
          card2: cards[1].to_string(),
        };
        vec.push(hole);
      }
    }
    vec
  }
}

#[derive(Debug, PartialEq)]
pub enum Action {
  Call(Player, f32, bool),
  /// is all-in
  Bet(Player, f32, bool),
  Raise(Player, f32, f32, bool), // raise from .. to ..
  Check(Player),
  Fold(Player),
  Leave(Player),
  UncalledBet(Player, f32),
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Player {
  pub name: String,
  pub position: u8,
  pub bank: f32,
}

#[derive(Default, Debug, PartialEq)]
pub struct Blind {
  pub player: Player,
  pub amount: f32,
}

#[derive(Default, Debug, PartialEq)]
pub struct End {
  pub pot: f32,
  pub winner: Player,
}
