use anyhow::Context;
use serde::Serialize;
use spin_sdk::{
    http::{Params, Request, Response},
    http_component, http_router,
    sqlite::{self, Connection},
};

#[http_component]
fn handle_voting_submission(req: Request) -> anyhow::Result<Response> {
    let router = http_router! {
        GET "/api/votingsubmissions" => get_voting_submissions,
        POST "/api/votingsubmissions/create" => create_voting_submission,
        PATCH "/api/votingsubmissions/:id/vote" => vote_on_submission,
        DELETE "/api/votingsubmissions/:id" => delete_voting_submission,
        _   "/*"             => |req, _params| {
            println!("No handler for {} {}", req.uri(), req.method());
            Ok(http::Response::builder()
                .status(http::StatusCode::NOT_FOUND)
                .body(Some(serde_json::json!({"error":"not_found"}).to_string().into()))
                .unwrap())
        }
    };
    router.handle(req)
}

#[derive(serde::Deserialize)]
struct GetParams {
    #[serde(default)]
    vote_count: Option<bool>,
}

#[derive(serde::Deserialize)]
struct CreateParams {
    description: String,
}

pub fn get_voting_submissions(req: Request, _params: Params) -> anyhow::Result<Response> {
    let query = req.uri().query().unwrap_or_default();
    let params: GetParams = serde_qs::from_str(query)?;

    let conn = Connection::open_default()?;
    let voting_submissions = conn
        .execute("SELECT * FROM votingsubmissions;", &[])?
        .rows()
        .map(|r| -> anyhow::Result<VotingSubmission> { r.try_into() })
        .collect::<anyhow::Result<Vec<VotingSubmission>>>()?;

    Ok(http::Response::builder()
        .status(http::StatusCode::OK)
        .body(Some(serde_json::to_vec(&voting_submissions)?.into()))
        .unwrap())
}

pub fn create_voting_submission(req: Request, _params: Params) -> anyhow::Result<Response> {
    let create: CreateParams = serde_json::from_slice(
        req.body()
            .as_ref()
            .map(|b| -> &[u8] { &*b })
            .unwrap_or_default(),
    )?;

    let params = [sqlite::ValueParam::Text(&create.description)];

    let conn = Connection::open_default()?;
    let response = &conn
        .execute(
            "INSERT INTO votingsubmissions (description) VALUES(?);",
            params.as_slice(),
        )?
        .rows;
    let id = match response.get(0) {
        Some(id) => id,
        None => anyhow::bail!("Expected a number, but got none."),
    };
        let voting_submission = VotingSubmission {
        id: id.get(0).unwrap(),
        description: create.description,
        vote_count: 0,
    };

    Ok(http::Response::builder()
        .status(http::StatusCode::OK)
        .body(Some(serde_json::to_vec(&voting_submission)?.into()))
        .unwrap())
}

#[derive(serde::Deserialize)]
struct VoteParams {
    vote_count: i32,
}

pub fn vote_on_submission(req: Request, params: Params) -> anyhow::Result<Response> {
    let id = params.get("id").unwrap();
    let vote: VoteParams = serde_json::from_slice(
        req.body()
            .as_ref()
            .map(|b| -> &[u8] { &*b })
            .unwrap_or_default(),
    )?;

    let params = [
        sqlite::ValueParam::Integer(vote.vote_count.into()), // Convert i32 to i64
        sqlite::ValueParam::Integer(id.parse().unwrap()),
    ];

    let conn = Connection::open_default()?;
    conn.execute(
        "UPDATE votingsubmissions SET vote_count = vote_count + (?) WHERE ID = (?);",
        params.as_slice(),
    )?;

    Ok(http::Response::builder().status(204).body(None).unwrap())
}

pub fn delete_voting_submission(_req: Request, params: Params) -> anyhow::Result<Response> {
    let id = params.get("id").unwrap();
    let params = [sqlite::ValueParam::Integer(id.parse().unwrap())];
    let conn = Connection::open_default()?;
    conn.execute("DELETE FROM votingsubmissions WHERE ID = (?);", params.as_slice())?;

    Ok(http::Response::builder().status(204).body(None).unwrap())
}

#[derive(Serialize)]
struct VotingSubmission {
    id: u32,
    description: String,
    vote_count: i32,
}

impl<'a> TryFrom<sqlite::Row<'a>> for VotingSubmission {
    type Error = anyhow::Error;
    fn try_from(row: sqlite::Row<'a>) -> std::result::Result<Self, Self::Error> {
        let id = row.get("id").context("row has no id")?;
        let description: &str = row.get("description").context("row has no description")?;
        let vote_count = row.get::<u32>("vote_count")
            .ok_or(anyhow::anyhow!("row has no vote_count"))?
            as i32; // Convert i64 to i32
        Ok(Self {
            id,
            description: description.to_owned(),
            vote_count,
        })
    }
}

