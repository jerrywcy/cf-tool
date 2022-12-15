#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
//! Reference: [Codeforces Official API Documentation - Return objects](https://codeforces.com/apiHelp/methods)

use color_eyre::{
    eyre::{bail, WrapErr},
    Result,
};
use lazy_static::lazy_static;
use reqwest::{Client, StatusCode};
use serde::Deserialize;

use crate::settings::SETTINGS;

use super::{
    objects::{
        BlogEntry, Comment, Contest, Hack, ProblemSet, RatingChange, RecentAction, Standings,
        Submission, User,
    },
    utils::{get_authorize, CFApiResponse, CFApiResponseStatus, CFApiUrl},
};

lazy_static! {
    static ref CLIENT: Client = Client::builder()
        .build()
        .wrap_err("Failed to build Reqwest client")
        .unwrap();
}

async fn request<T>(url: String) -> Result<T>
where
    T: for<'a> Deserialize<'a>,
{
    let request_error_message = format!("Error occured when making a POST request");
    let response = CLIENT
        .get(url)
        .send()
        .await
        .wrap_err(request_error_message)?;
    let status_code = response.status();
    if status_code != StatusCode::OK {
        bail!("Server returned status: {status_code}");
    }
    let json_error_message = format!(
        "Failed to parse\n{:#?}\ninto CFApiResponse<Vec<Comment>>",
        response
    );
    let response = response
        .json::<CFApiResponse<T>>()
        .await
        .wrap_err(json_error_message)?;
    match response.status {
        CFApiResponseStatus::OK => match response.result {
            Some(result) => Ok(result),
            None => bail!("Server returned status OK with no result"),
        },
        CFApiResponseStatus::FAILED => match response.comment {
            Some(comment) => {
                bail!("Server returned status FAILED with the following comment:\n{comment}")
            }
            None => bail!("Server return status FAILED with no comment"),
        },
    }
}

async fn request_smart<T>(url: &mut CFApiUrl, must_authorize: bool) -> Result<T>
where
    T: for<'a> Deserialize<'a>,
{
    if must_authorize {
        let (key, secret) = get_authorize(&SETTINGS.key, &SETTINGS.secret)?;
        Ok(request(url.authorize(key, secret)).await?)
    } else {
        match get_authorize(&SETTINGS.key, &SETTINGS.secret) {
            Ok((key, secret)) => Ok(request(url.authorize(key, secret))
                .await
                .unwrap_or(request(url.into_url()).await?)),
            Err(_) => Ok(request(url.into_url()).await?),
        }
    }
}

/// Returns a list of comments to the specified blog entry.
///
/// | Parameter	| Description |
/// | --------- | ----------- |
/// | **blogEntryId** (Required) | Id of the blog entry. It can be seen in blog entry URL. For example: [/blog/entry/**79**](https://codeforces.com/blog/entry/79) |
///
/// **Return value**: A list of [Comment](https://codeforces.com/apiHelp/objects#Comment) objects.
///
/// **Example**: [https://codeforces.com/api/blogEntry.comments?blogEntryId=79](https://codeforces.com/api/blogEntry.comments?blogEntryId=79)
pub async fn blogEntry_comments(blogEntryId: i32) -> Result<Vec<Comment>> {
    request_smart::<Vec<Comment>>(
        CFApiUrl::new("blogEntry.comments").add_required_parameter("blogEntryId", blogEntryId),
        false,
    )
    .await
}

/// Returns blog entry.
///
/// | Parameter | Description |
/// | --- | --- |
/// | **blogEntryId** (Required) | Id of the blog entry. It can be seen in blog entry URL. For example: [/blog/entry/**79**](https://codeforces.com/blog/entry/79) |
///
/// **Return value**: Returns a [BlogEntry](https://codeforces.com/apiHelp/objects#BlogEntry) object in full version.
///
/// **Example**: [https://codeforces.com/api/blogEntry.view?blogEntryId=79](https://codeforces.com/api/blogEntry.view?blogEntryId=79)
pub async fn blogEntry_view(blogEntryId: i32) -> Result<BlogEntry> {
    request_smart::<BlogEntry>(
        CFApiUrl::new("blogEntry.view").add_required_parameter("blogEntryId", blogEntryId),
        false,
    )
    .await
}

/// Returns list of hacks in the specified contests. Full information about hacks is available only after some time after the contest end. During the contest user can see only own hacks.
///
/// | Parameter | Description |
/// | --- | --- |
/// | **contestId** (Required) | Id of the contest. It is **not** the round number. It can be seen in contest URL. For example: [/contest/**566**/status](https://codeforces.com/contest/566/status) |
///
/// **Return value**: Returns a list of [Hack](https://codeforces.com/apiHelp/objects#Hack) objects.
///
/// **Example**: [https://codeforces.com/api/contest.hacks?contestId=566](https://codeforces.com/api/contest.hacks?contestId=566)
pub async fn contest_hacks(contestId: i32) -> Result<Vec<Hack>> {
    request_smart::<Vec<Hack>>(
        CFApiUrl::new("contest.hacks").add_required_parameter("contestId", contestId),
        false,
    )
    .await
}

/// Returns information about all available contests.
///
/// | Parameter | Description |
/// | --- | --- |
/// | **gym** | Boolean. If true — than gym contests are returned. Otherwide, regular contests are returned. |
///
/// **Return value**: Returns a list of [Contest](https://codeforces.com/apiHelp/objects#Contest) objects. If this method is called not anonymously, then all available contests for a calling user will be returned too, including mashups and private gyms.
///
/// **Example**: [https://codeforces.com/api/contest.list?gym=true](https://codeforces.com/api/contest.list?gym=true)
pub async fn contest_list(gym: Option<bool>) -> Result<Vec<Contest>> {
    request_smart::<Vec<Contest>>(
        CFApiUrl::new("contest.list").add_parameter("gym", gym),
        false,
    )
    .await
}

/// Returns rating changes after the contest.
///
/// | Parameter | Description |
/// | --- | --- |
/// | **contestId** (Required) | Id of the contest. It is **not** the round number. It can be seen in contest URL. For example: [/contest/**566**/status](https://codeforces.com/contest/566/status) |
///
/// **Return value**: Returns a list of [RatingChange](https://codeforces.com/apiHelp/objects#RatingChange) objects.
///
/// **Example**: [https://codeforces.com/api/contest.ratingChanges?contestId=566](https://codeforces.com/api/contest.ratingChanges?contestId=566)
pub async fn contest_ratingChanges(contestId: i32) -> Result<Vec<RatingChange>> {
    request_smart::<Vec<RatingChange>>(
        CFApiUrl::new("contest.hacks").add_required_parameter("contestId", contestId),
        false,
    )
    .await
}

/// Returns the description of the contest and the requested part of the standings.
///
/// | Parameter | Description |
/// | --- | --- |
/// | **contestId** (Required) | Id of the contest. It is **not** the round number. It can be seen in contest URL. For example: [/contest/**566**/status](https://codeforces.com/contest/566/status) |
/// | **from** | 1-based index of the standings row to start the ranklist. |
/// | **count** | Number of standing rows to return. |
/// | **handles** | Semicolon-separated list of handles. No more than 10000 handles is accepted. |
/// | **room** | If specified, than only participants from this room will be shown in the result. If not — all the participants will be shown. |
/// | **showUnofficial** | If true than all participants (virtual, out of competition) are shown. Otherwise, only official contestants are shown. |
///
/// **Return value**: Returns object with three fields: "contest", "problems" and "rows". Field "contest" contains a [Contest](https://codeforces.com/apiHelp/objects#Contest) object. Field "problems" contains a list of [Problem](https://codeforces.com/apiHelp/objects#Problem) objects. Field "rows" contains a list of [RanklistRow](https://codeforces.com/apiHelp/objects#RanklistRow) objects.
///
/// **Example**: [https://codeforces.com/api/contest.standings?contestId=566&from=1&count=5&showUnofficial=true](https://codeforces.com/api/contest.standings?contestId=566&from=1&count=5&showUnofficial=true)
pub async fn contest_standings(
    contestId: i32,
    from: Option<i32>,
    count: Option<i32>,
    handles: Option<Vec<String>>,
    room: Option<i32>,
    showUnofficial: Option<bool>,
) -> Result<Standings> {
    request_smart::<Standings>(
        CFApiUrl::new("contest.standings")
            .add_required_parameter("contestId", contestId)
            .add_parameter("from", from)
            .add_parameter("count", count)
            .add_parameter(
                "handles",
                match handles {
                    Some(handles) => Some(handles.join(";")),
                    None => None,
                },
            )
            .add_parameter("room", room)
            .add_parameter("showUnofficial", showUnofficial),
        false,
    )
    .await
}

/// Returns submissions for specified contest. Optionally can return submissions of specified user.
///
/// | Parameter | Description |
/// | --- | --- |
/// | **contestId** (Required) | Id of the contest. It is **not** the round number. It can be seen in contest URL. For example: [/contest/**566**/status](https://codeforces.com/contest/566/status) |
/// | **handle** | Codeforces user handle. |
/// | **from** | 1-based index of the first submission to return. |
/// | **count** | Number of returned submissions. |
///
/// **Return value**: Returns a list of [Submission](https://codeforces.com/apiHelp/objects#Submission) objects, sorted in decreasing order of submission id.
///
/// **Example**: [https://codeforces.com/api/contest.status?contestId=566&from=1&count=10](https://codeforces.com/api/contest.status?contestId=566&from=1&count=10)
pub async fn contest_status(
    contestId: i32,
    handle: Option<String>,
    from: Option<i32>,
    count: Option<i32>,
) -> Result<Vec<Submission>> {
    request_smart::<Vec<Submission>>(
        CFApiUrl::new("contest.status")
            .add_required_parameter("contestId", contestId)
            .add_parameter("handle", handle)
            .add_parameter("from", from)
            .add_parameter("count", count),
        false,
    )
    .await
}

/// Returns all problems from problemset. Problems can be filtered by tags.
///
/// | Parameter | Description |
/// | --- | --- |
/// | **tags** | Semicilon-separated list of tags. |
/// | **problemsetName** | Custom problemset's short name, like 'acmsguru' |
///
/// **Return value**: Returns two lists. List of [Problem](https://codeforces.com/apiHelp/objects#Problem) objects and list of [ProblemStatistics](https://codeforces.com/apiHelp/objects#ProblemStatistics) objects.
///
/// **Example**: [https://codeforces.com/api/problemset.problems?tags=implementation](https://codeforces.com/api/problemset.problems?tags=implementation)
pub async fn problemset_problems(
    tags: Option<Vec<String>>,
    problemsetName: Option<String>,
) -> Result<ProblemSet> {
    request_smart::<ProblemSet>(
        CFApiUrl::new("problemset.problems")
            .add_parameter(
                "tags",
                match tags {
                    Some(tags) => Some(tags.join(";")),
                    None => None,
                },
            )
            .add_parameter("problemsetName", problemsetName),
        false,
    )
    .await
}

/// Returns recent submissions.
///
/// | Parameter | Description |
/// | --- | --- |
/// | **count** (Required) | Number of submissions to return. Can be up to 1000. |
/// | **problemsetName** | Custom problemset's short name, like 'acmsguru' |
///
/// **Return value**: Returns a list of [Submission](https://codeforces.com/apiHelp/objects#Submission) objects, sorted in decreasing order of submission id.
///
/// **Example**: [https://codeforces.com/api/problemset.recentStatus?count=10](https://codeforces.com/api/problemset.recentStatus?count=10)
pub async fn problemset_recentStatus(
    count: i32,
    problemsetName: Option<String>,
) -> Result<Vec<Submission>> {
    request_smart::<Vec<Submission>>(
        CFApiUrl::new("problemset.problems")
            .add_required_parameter("count", count)
            .add_parameter("problemsetName", problemsetName),
        false,
    )
    .await
}

/// Returns recent actions.
///
/// | Parameter | Description |
/// | --- | --- |
/// | **maxCount** (Required) | Number of recent actions to return. Can be up to 100. |
///
/// **Return value**: Returns a list of [RecentAction](https://codeforces.com/apiHelp/objects#RecentAction) objects.
///
/// **Example**: [https://codeforces.com/api/recentActions?maxCount=30](https://codeforces.com/api/recentActions?maxCount=30)
pub async fn recentActions(maxCount: i32) -> Result<Vec<RecentAction>> {
    request_smart::<Vec<RecentAction>>(
        CFApiUrl::new("recentActions").add_required_parameter("maxCount", maxCount),
        false,
    )
    .await
}

/// Returns a list of all user's blog entries.
///
/// | Parameter | Description |
/// | --- | --- |
/// | **handle** (Required) | Codeforces user handle. |
///
/// **Return value**: A list of [BlogEntry](https://codeforces.com/apiHelp/objects#BlogEntry) objects in short form.
///
/// **Example**: [https://codeforces.com/api/user.blogEntries?handle=Fefer\_Ivan](https://codeforces.com/api/user.blogEntries?handle=Fefer_Ivan)
pub async fn user_blogEntries(handle: String) -> Result<Vec<BlogEntry>> {
    request_smart::<Vec<BlogEntry>>(
        CFApiUrl::new("u Note that this is the representation you get by default if you return an error from fn main instead of printing it explicitly yourser.blogEntries").add_required_parameter("handle", handle),
        false,
    )
    .await
}

/// Returns authorized user's friends. Using this method requires authorization.
///
/// | Parameter | Description |
/// | --- | --- |
/// | **onlyOnline** | Boolean. If true — only online friends are returned. Otherwise, all friends are returned. |
///
/// **Return value**: Returns a list of strings — users' handles.
///
/// **Example**: [https://codeforces.com/api/user.friends?onlyOnline=true](https://codeforces.com/api/user.friends?onlyOnline=true)
pub async fn user_friends(onlyOnline: Option<bool>) -> Result<Vec<String>> {
    request_smart::<Vec<String>>(
        CFApiUrl::new("user.friends").add_parameter("onlyOnline", onlyOnline),
        false,
    )
    .await
}

/// Returns information about one or several users.
///
/// | Parameter | Description |
/// | --- | --- |
/// | **handles** (Required) | Semicolon-separated list of handles. No more than 10000 handles is accepted. |
///
/// **Return value**: Returns a list of [User](https://codeforces.com/apiHelp/objects#User) objects for requested handles.
///
/// **Example**: [https://codeforces.com/api/user.info?handles=DmitriyH;Fefer\_Ivan](https://codeforces.com/api/user.info?handles=DmitriyH;Fefer_Ivan)
pub async fn user_info(handles: Vec<String>) -> Result<Vec<User>> {
    request_smart::<Vec<User>>(
        CFApiUrl::new("user.info").add_required_parameter("handles", handles.join(";")),
        false,
    )
    .await
}

/// Returns the list users who have participated in at least one rated contest.
///
/// | Parameter | Description |
/// | --- | --- |
/// | **activeOnly** | Boolean. If true then only users, who participated in rated contest during the last month are returned. Otherwise, all users with at least one rated contest are returned. |
/// | **includeRetired** | Boolean. If true, the method returns all rated users, otherwise the method returns only users, that were online at last month. |
/// | **contestId** | Id of the contest. It is **not** the round number. It can be seen in contest URL. For example: [/contest/**566**/status](https://codeforces.com/contest/566/status) |
///
/// **Return value**: Returns a list of [User](https://codeforces.com/apiHelp/objects#User) objects, sorted in decreasing order of rating.
///
/// **Example**: [https://codeforces.com/api/user.ratedList?activeOnly=true&includeRetired=false](https://codeforces.com/api/user.ratedList?activeOnly=true&includeRetired=false)
pub async fn user_ratedList(
    activeOnly: Option<bool>,
    includeRetired: Option<bool>,
    contestId: Option<i32>,
) -> Result<Vec<User>> {
    request_smart::<Vec<User>>(
        CFApiUrl::new("user.ratedList")
            .add_parameter("activeOnly", activeOnly)
            .add_parameter("includeRetired", includeRetired)
            .add_parameter("contestId", contestId),
        false,
    )
    .await
}

/// Returns rating history of the specified user.
///
/// | Parameter | Description |
/// | --- | --- |
/// | **handle** (Required) | Codeforces user handle. |
///
/// **Return value**: Returns a list of [RatingChange](https://codeforces.com/apiHelp/objects#RatingChange) objects for requested user.
///
/// **Example**: [https://codeforces.com/api/user.rating?handle=Fefer\_Ivan](https://codeforces.com/api/user.rating?handle=Fefer_Ivan)
pub async fn user_rating(handle: String) -> Result<Vec<RatingChange>> {
    request_smart(
        CFApiUrl::new("user.rating").add_required_parameter("handle", handle),
        false,
    )
    .await
}

/// Returns submissions of specified user.
///
/// | Parameter | Description |
/// | --- | --- |
/// | **handle** (Required) | Codeforces user handle. |
/// | **from** | 1-based index of the first submission to return. |
/// | **count** | Number of returned submissions. |
///
/// **Return value**: Returns a list of [Submission](https://codeforces.com/apiHelp/objects#Submission) objects, sorted in decreasing order of submission id.
///
/// **Example**: [https://codeforces.com/api/user.status?handle=Fefer\_Ivan&from=1&count=10](https://codeforces.com/api/user.status?handle=Fefer_Ivan&from=1&count=10)
pub async fn user_status(
    handle: String,
    from: Option<i32>,
    count: Option<i32>,
) -> Result<Vec<Submission>> {
    request_smart::<Vec<Submission>>(
        CFApiUrl::new("user.status")
            .add_required_parameter("handle", handle)
            .add_parameter("from", from)
            .add_parameter("count", count),
        false,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    mod problemset_problems_test {
        use super::problemset_problems;

        #[tokio::test]
        async fn no_panic() {
            problemset_problems(None, None).await.unwrap();
        }
    }

    mod contest_list_test {
        use super::contest_list;

        #[tokio::test]
        async fn no_panic() {
            contest_list(None).await.unwrap();
        }
    }
}
