// External imports
use bson::{doc, Document, oid::ObjectId};
use mongodb::results::{DeleteResult, UpdateResult, InsertOneResult};
use mongodb::{error::Error, Collection};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use chrono::serde::ts_seconds;
// External constructors
extern crate serde;
extern crate serde_json;

// User Model
#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub username: String,
    pub password: String,
    pub wins: i32,
    pub losses: i32,
}

// Match Model
#[derive(Serialize, Deserialize, Debug)]
pub struct Match {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub winner: String,
    pub loser: String,
    pub game_type: i32, // 0 = Connect4; 1 = TooT OttO
    #[serde(with = "ts_seconds")]
    pub time: chrono::DateTime<Utc>, //nessary field
}

// Reference colection clone
#[derive(Clone)]
pub struct ApiService {
    users: Collection,
    match_history: Collection,
}

// User to document
fn user_to_document(user: &User) -> Document {
    let User {
        username,
        password,
        wins,
        losses,
        ..
    } = user;
    doc! {
        "username": username,
        "password": password,
        "wins": wins,
        "losses": losses,
    }
}

// Match to document
fn match_to_document(match_data: &Match) -> Document {
    let Match {
        winner,
        loser,
        game_type,
        ..
    } = match_data;
    doc! {
        "winner": winner,
        "loser": loser,
        "game_type": game_type,
        "time": Utc::now()
    }
}

// Functions with queries to Mongo
impl ApiService {
    //Constructor, user collection and match collection
    pub fn new(users: Collection, match_history: Collection) -> ApiService {
        ApiService { users, match_history }
    }

    // Insert new User
    pub fn create_user(&self, _user: &User) -> Result<InsertOneResult, Error> {
        self.users.insert_one(user_to_document(_user), None)
    }

    // Insert new Match
    pub fn create_match(&self, _match: &Match) -> Result<InsertOneResult, Error> {
        self.match_history.insert_one(match_to_document(_match), None)
    }

    // Get user based on username
    pub fn get_user(&self, username: &String) -> Result<Option<bson::ordered::OrderedDocument>, mongodb::error::Error> {
        let document = self.users.find_one(doc! {
            "username": username,
        }, None).ok().expect("Failed to execute find!");
        Ok(document)
    }

    // Get matches based on username
    pub fn get_matches(&self, username: &String) -> Result<Vec<bson::ordered::OrderedDocument>, mongodb::error::Error> {
        let cursor = self.match_history.find(doc! {
            "$or": [
                {"winner": username.to_owned()},
                {"loser": username.to_owned()}
            ],
        }, None).ok().expect("Failed to execute find!");
        let docs: Vec<_> = cursor.map(|doc| doc.unwrap()).collect();
        Ok(docs)
    }

    // Update user based on username
    pub fn update_user(&self, _user: &User, _username: &String) -> Result<UpdateResult, Error> {
        self.users.update_one(doc! {"username": _username}, user_to_document(_user), None)
    }

    // Delete user
    pub fn delete_user(&self, _user: &User) -> Result<DeleteResult, Error> {
        self.users.delete_one(doc! { "username": _user.username.to_owned() }, None)
    }

    // Get all matches
    pub fn get_all_matches(&self) -> Result<Vec<bson::ordered::OrderedDocument>, mongodb::error::Error> {
        let cursor = self.match_history.find(doc! {}, None).ok().expect("Failed to execute find!");
        let docs: Vec<_> = cursor.map(|doc| doc.unwrap()).collect();
        Ok(docs)
    }

    // Update a match based on id
    pub fn update_match(&self, _match_data: &Match, _id: &String) -> Result<UpdateResult, Error>{
        let match_id = bson::oid::ObjectId::with_string(_id).unwrap();
        self.match_history.update_one(doc! {"_id": match_id}, match_to_document(_match_data), None)
    }

    // Delete a match based on id
    pub fn delete_match(&self, _id: &String) -> Result<DeleteResult, Error> {
        let match_id = bson::oid::ObjectId::with_string(_id).unwrap();
        self.match_history.delete_one(doc! {"_id": match_id}, None)
    }
}
