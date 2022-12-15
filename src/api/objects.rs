#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
//! Reference: [Codeforces Official API Documentation - Return objects](https://codeforces.com/apiHelp/objects)

use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// Represents a Codeforces user.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    /// String. Codeforces user handle.
    pub handle: String,
    /// String. Shown only if user allowed to share his contact info.
    pub email: Option<String>,
    /// String. User id for VK social network. Shown only if user allowed to share his contact info.
    pub vkId: Option<String>,
    /// String. Shown only if user allowed to share his contact info.
    pub openId: Option<String>,
    /// String. Localized. Can be absent.
    pub firstName: Option<String>,
    /// String. Localized. Can be absent.
    pub lastName: Option<String>,
    /// String. Localized. Can be absent.
    pub country: Option<String>,
    /// String. Localized. Can be absent.
    pub city: Option<String>,
    /// String. Localized. Can be absent.
    pub organization: Option<String>,
    /// Integer. User contribution.
    pub contribution: i32,
    /// Integer. User contribution.
    pub rank: String,
    /// Integer.
    pub rating: i32,
    /// String. Localized.
    pub maxRank: String,
    /// Integer.
    pub maxRating: i32,
    /// Integer. Time, when user was last seen online, in unix format.
    pub lastOnlineTimeSeconds: i64,
    /// Integer. Time, when user was last seen online, in unix format.
    pub registrationTimeSeconds: i64,
    /// Integer. Amount of users who have this user in friends.
    pub friendOfCount: i32,
    /// String. User's avatar URL.
    pub avatar: String,
    /// String. User's title photo URL.
    pub titlePhoto: String,
}

/// Represents a Codeforces blog entry. May be in either short or full version.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlogEntry {
    /// Integer
    pub id: i32,
    /// String. Original locale of the blog entry.
    pub originalLocale: String,
    /// Integer. Time, when blog entry was created, in unix format.
    pub creationTimeSeconds: i64,
    /// String. Author user handle.
    pub authorHandle: String,
    /// String. Localized.
    pub title: String,
    /// String. Localized. Not included in short version.
    pub content: Option<String>,
    /// String.
    pub locale: String,
    /// Integer. Time, when blog entry has been updated, in unix format.
    pub modificationTimeSeconds: i64,
    /// Boolean. If true, you can view any specific revision of the blog entry.
    pub allowViewHistory: bool,
    /// String list.
    pub tags: Vec<String>,
    /// Integer.
    pub rating: i32,
}

/// Represents a comment.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Comment {
    /// Integer.
    pub id: i32,
    /// Integer. Time, when comment was created, in unix format.
    pub creationTimeSeconds: i64,
    /// String.
    pub commentatorHandle: String,
    /// String.
    pub locale: String,
    /// String.
    pub text: String,
    /// Integer. Can be absent.
    pub parentCommentId: Option<i32>,
    /// Integer.
    pub rating: i32,
}

/// Represents a recent action.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecentAction {
    /// Integer. Action time, in unix format.
    pub timeSeconds: i64,
    /// [`BlogEntry`] object in short form. Can be absent.
    pub blogEntry: Option<BlogEntry>,
    /// [`Comment`] object. Can be absent.
    pub comment: Option<Comment>,
}

/// Represents a participation of user in rated contest.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RatingChange {
    /// Integer.
    pub contestId: i32,
    /// String. Localized.
    pub contestName: String,
    /// String. Codeforces user handle.
    pub handle: String,
    /// Integer. Place of the user in the contest. This field contains user rank on the moment of rating update. If afterwards rank changes (e.g. someone get disqualified), this field will not be update and will contain old rank.
    pub rank: i32,
    /// Integer. Time, when rating for the contest was update, in unix-format.
    pub ratingUpdateTimeSeconds: i64,
    /// Integer. User rating before the contest.
    pub oldRating: i32,
    /// Integer. User rating after the contest.
    pub newRating: i32,
}

/// Scoring system used for a [`Contest`].
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ContestType {
    CF,
    IOI,
    ICPC,
}

/// The phase a [`Contest`] is in.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ContestPhase {
    BEFORE,
    CODING,
    PENDING_SYSTEM_TEST,
    SYSTEM_TEST,
    FINISHED,
}

/// Represents a contest on Codeforces.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Contest {
    /// Integer.
    pub id: i32,
    /// String. Localized.
    pub name: String,
    /// Enum: CF, IOI, ICPC. Scoring system used for the contest.
    pub r#type: ContestType,
    /// Enum: BEFORE, CODING, PENDING_SYSTEM_TEST, SYSTEM_TEST, FINISHED.
    pub phase: ContestPhase,
    /// Boolean. If true, then the ranklist for the contest is frozen and shows only submissions, created before freeze.
    pub frozen: bool,
    /// Integer. Duration of the contest in seconds.
    pub durationSeconds: u64,
    /// Integer. Can be absent. Contest start time in unix format.
    pub startTimeSeconds: Option<i64>,
    /// Integer. Can be absent. Number of seconds, passed after the start of the contest. Can be negative.
    pub relativeTimeSecods: Option<i64>,
    /// String. Can be absent. Handle of the user, how created the contest.
    pub preparedBy: Option<String>,
    /// String. Can be absent. URL for contest-related website.
    pub websiteUrl: Option<String>,
    /// String. Localized. Can be absent.
    pub description: Option<String>,
    /// Integer. Can be absent. From 1 to 5. Larger number means more difficult problems.
    pub difficulty: Option<i32>,
    /// String. Localized. Can be absent. Human-readable type of the contest from the following categories: Official ICPC Contest, Official School Contest, Opencup Contest, School/University/City/Region Championship, Training Camp Contest, Official International Personal Contest, Training Contest.
    pub kind: Option<String>,
    /// String. Localized. Can be absent. Name of the Region for official ICPC contests.
    pub icpcRegion: Option<String>,
    /// String. Localized. Can be absent.
    pub country: Option<String>,
    /// String. Localized. Can be absent.
    pub city: Option<String>,
    /// String. Can be absent.
    pub season: Option<String>,
}

/// Type of participant in a [`Party`].
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ParticipantType {
    CONTESTANT,
    PRACTICE,
    VIRTUAL,
    MANAGER,
    OUT_OF_COMPETITION,
}

/// Represents a party, participating in a contest.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Party {
    /// Integer. Can be absent. Id of the contest, in which party is participating.
    pub contestId: Option<i32>,
    /// List of [`Member`] objects. Members of the party.
    pub members: Vec<Member>,
    /// Enum: CONTESTANT, PRACTICE, VIRTUAL, MANAGER, OUT_OF_COMPETITION.
    pub participantType: ParticipantType,
    /// Integer. Can be absent. If party is a team, then it is a unique team id. Otherwise, this field is absent.
    pub teamId: Option<i32>,
    /// String. Localized. Can be absent. If party is a team or ghost, then it is a localized name of the team. Otherwise, it is absent.
    pub teamName: Option<String>,
    /// Boolean. If true then this party is a ghost. It participated in the contest, but not on Codeforces. For example, Andrew Stankevich Contests in Gym has ghosts of the participants from Petrozavodsk Training Camp.
    pub ghost: bool,
    /// Integer. Can be absent. Room of the party. If absent, then the party has no room.
    pub room: Option<i32>,
    /// Integer. Can be absent. Time, when this party started a contest.
    pub startTimeSecons: Option<i64>,
}

/// Represents a member of a party.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Member {
    /// String. Codeforces user handle.
    pub handle: String,
    /// String. Can be absent. User's name if available.
    pub name: Option<String>,
}

/// Type of a [`Problem`].
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProblemType {
    PROGRAMMING,
    QUESTION,
}

/// Represents a problem.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Problem {
    /// Integer. Can be absent. Id of the contest, containing the problem.
    pub contestId: Option<i32>,
    /// String. Can be absent. Short name of the problemset the problem belongs to.
    pub problemsetName: Option<String>,
    /// String. Usually, a letter or letter with digit(s) indicating the problem index in a contest.
    pub index: String,
    /// String. Localized.
    pub name: String,
    /// Enum: PROGRAMMING, QUESTION.
    pub r#type: ProblemType,
    /// Floating point number. Can be absent. Maximum amount of points for the problem.
    pub points: Option<f32>,
    /// Integer. Can be absent. Problem rating (difficulty).
    pub rating: Option<i32>,
    /// String list. Problem tags.
    pub tags: Vec<String>,
}

/// Represents a statistic data about a problem.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProblemStatistics {
    /// Integer. Can be absent. Id of the contest, containing the problem.
    pub contestId: Option<i32>,
    /// String. Usually, a letter or letter with digit(s) indicating the problem index in a contest.
    pub index: String,
    /// Integer. Number of users, who solved the problem.
    pub solvedCount: i32,
}

/// Verdict of a [`Submission`].
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SubmissionVerdict {
    FAILED,
    OK,
    PARTIAL,
    COMPILATION_ERROR,
    RUNTIME_ERROR,
    WRONG_ANSWER,
    PRESENTATION_ERROR,
    TIME_LIMIT_EXCEEDED,
    MEMORY_LIMIT_EXCEEDED,
    IDLENESS_LIMIT_EXCEEDED,
    SECURITY_VIOLATED,
    CRASHED,
    INPUT_PREPARATION_CRASHED,
    CHALLENGED,
    SKIPPED,
    TESTING,
    REJECTED,
}

impl Display for SubmissionVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SubmissionVerdict::FAILED => "Failed",
                SubmissionVerdict::OK => "Accepted",
                SubmissionVerdict::PARTIAL => "Partial",
                SubmissionVerdict::COMPILATION_ERROR => "Compilation Error",
                SubmissionVerdict::RUNTIME_ERROR => "Runtime Error",
                SubmissionVerdict::WRONG_ANSWER => "Wrong Answer",
                SubmissionVerdict::PRESENTATION_ERROR => "Presentation Error",
                SubmissionVerdict::TIME_LIMIT_EXCEEDED => "Time Limit Exceeded",
                SubmissionVerdict::MEMORY_LIMIT_EXCEEDED => "Memory Limit Exceeded",
                SubmissionVerdict::IDLENESS_LIMIT_EXCEEDED => "Idleness Limit Exceeded",
                SubmissionVerdict::SECURITY_VIOLATED => "Security Violated",
                SubmissionVerdict::CRASHED => "Crashed",
                SubmissionVerdict::INPUT_PREPARATION_CRASHED => "Input Preparation Crashed",
                SubmissionVerdict::CHALLENGED => "Challenged",
                SubmissionVerdict::SKIPPED => "Skipped",
                SubmissionVerdict::TESTING => "Testing",
                SubmissionVerdict::REJECTED => "Rejected",
            }
        )
    }
}

/// Testset used for judging a [`Submission`].
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SubmissionTestset {
    SAMPLES,
    PRETESTS,
    TESTS,
    CHALLENGES,
    TESTS1,
    TESTS2,
    TESTS3,
    TESTS4,
    TESTS5,
    TESTS6,
    TESTS7,
    TESTS8,
    TESTS9,
    TESTS10,
}

/// Represents a submission.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Submission {
    /// Integer.
    pub id: i32,
    /// Integer. Can be absent.
    pub contestId: Option<i32>,
    /// Integer. Time, when submission was created, in unix-format.
    pub creationTimeSeconds: i64,
    /// Integer. Number of seconds, passed after the start of the contest (or a virtual start for virtual parties), before the submission.
    pub relativeTimeSeconds: i64,
    /// Problem object.
    pub problem: Problem,
    /// Party object.
    pub author: Party,
    /// String.
    pub programmingLanguage: String,
    /// Enum: FAILED, OK, PARTIAL, COMPILATION_ERROR, RUNTIME_ERROR, WRONG_ANSWER, PRESENTATION_ERROR, TIME_LIMIT_EXCEEDED, MEMORY_LIMIT_EXCEEDED, IDLENESS_LIMIT_EXCEEDED, SECURITY_VIOLATED, CRASHED, INPUT_PREPARATION_CRASHED, CHALLENGED, SKIPPED, TESTING, REJECTED. Can be absent.
    pub verdict: Option<SubmissionVerdict>,
    /// Enum: SAMPLES, PRETESTS, TESTS, CHALLENGES, TESTS1, ..., TESTS10. Testset used for judging the submission.
    pub testset: SubmissionTestset,
    /// Integer. Number of passed tests.
    pub passedTestCount: i32,
    /// Integer. Maximum time in milliseconds, consumed by solution for one test.
    pub timeConsumedMillis: u64,
    /// Integer. Maximum memory in bytes, consumed by solution for one test.
    pub memoryConsumedBytes: u64,
    /// Floating point number. Can be absent. Number of scored points for IOI-like contests.
    pub points: Option<f32>,
}

/// Verdict of a [`Hack`].
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum HackVerdict {
    HACK_SUCCESSFUL,
    HACK_UNSUCCESSFUL,
    INVALID_INPUT,
    GENERATOR_INCOMPILABLE,
    GENERATOR_CRASHED,
    IGNORED,
    TESTING,
    OTHER,
}

/// Judge protocol used for a [`Hack`].
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HackJudgeProtocol {
    /// If manual is "true" then test for the hack was entered manually.
    pub manual: bool,
    /// Human-readable description of judge protocol.
    pub protocol: String,
    /// Human-readable description of hack verdict.
    pub verdict: String,
}

/// Represents a hack, made during Codeforces Round.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Hack {
    /// Integer.
    pub id: i32,
    /// Integer. Hack creation time in unix format.
    pub creationTimeSeconds: i64,
    /// [`Party`] object.
    pub hacker: Party,
    /// [`Party`] object.
    pub defender: Party,
    /// Enum: HACK_SUCCESSFUL, HACK_UNSUCCESSFUL, INVALID_INPUT, GENERATOR_INCOMPILABLE, GENERATOR_CRASHED, IGNORED, TESTING, OTHER. Can be absent.
    pub verdict: Option<HackVerdict>,
    /// [`Problem`] object. Hacked problem.
    pub problem: Problem,
    /// String. Can be absent.
    pub test: Option<String>,
    /// Object with three fields: "manual", "protocol" and "verdict". Field manual can have values "true" and "false". If manual is "true" then test for the hack was entered manually. Fields "protocol" and "verdict" contain human-readable description of judge protocol and hack verdict. Localized. Can be absent.
    pub judgeProtocol: Option<HackJudgeProtocol>,
}

/// Represents a ranklist row.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RanklistRow {
    /// [`Party`] object. Party that took a corresponding place in the contest.
    pub party: Party,
    /// Integer. Party place in the contest.
    pub rank: i32,
    /// Floating point number. Total amount of points, scored by the party.
    pub points: f32,
    /// Integer. Total penalty (in ICPC meaning) of the party.
    pub penalty: i32,
    /// Integer.
    pub successfulHackCount: i32,
    /// Integer.
    pub unsuccessfulHackCount: i32,
    /// List of [`ProblemResult`] objects. Party results for each problem. Order of the problems is the same as in "problems" field of the returned object.
    pub problemResults: Vec<ProblemResult>,
    /// Integer. For IOI contests only. Time in seconds from the start of the contest to the last submission that added some points to the total score of the party. Can be absent.
    pub lastSubmissionTimeSeconds: Option<i64>,
}

/// Type of a [`ProblemResult`].
/// If type is PRELIMINARY then points can decrease (if, for example, solution will fail during system test). Otherwise, party can only increase points for this problem by submitting better solutions.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProblemResultType {
    PRELIMINARY,
    FINAL,
}

/// Represents a submissions results of a party for a problem.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProblemResult {
    /// Floating point number.
    pub points: f32,
    /// Integer. Penalty (in ICPC meaning) of the party for this problem. Can be absent.
    pub penalty: Option<i32>,
    /// Integer. Number of incorrect submissions.
    pub rejectedAttemptCount: i32,
    /// Enum: PRELIMINARY, FINAL. If type is PRELIMINARY then points can decrease (if, for example, solution will fail during system test). Otherwise, party can only increase points for this problem by submitting better solutions.
    pub r#type: ProblemResultType,
    /// Integer. Number of seconds after the start of the contest before the submission, that brought maximal amount of points for this problem. Can be absent.
    pub bestSubmissionTimeSeconds: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Standings {
    pub contest: Contest,
    pub problems: Vec<Problem>,
    pub rows: Vec<RanklistRow>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProblemSet {
    pub problems: Vec<Problem>,
    pub problemStatistics: Vec<ProblemStatistics>,
}
