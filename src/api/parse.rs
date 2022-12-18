use color_eyre::{eyre::Context, Result};
use nipper::Document;

use super::methods::CLIENT;

#[derive(Debug, Clone)]
pub struct TestCase {
    pub input: String,
    pub answer: String,
}

pub async fn parse_testcase(url: String) -> Result<Vec<TestCase>> {
    let response = CLIENT
        .get(url)
        .send()
        .await
        .wrap_err("Error occured when making a GET request")?;
    let status_code = response.status();
    let text_error_message = format!(
        "Server returned status: {status_code}.\n\nFailed to parse\n{response:#?}\ninto text"
    );
    let response = response.text().await.wrap_err(text_error_message)?;

    let document = Document::from(&response);
    let inputs: Vec<String> = document
        .select("div.input")
        .iter()
        .map(|input| input.select("pre"))
        .map(|input| String::from(input.text()))
        .collect();
    let outputs: Vec<String> = document
        .select("div.output")
        .iter()
        .map(|output| output.select("pre"))
        .map(|output| String::from(output.text()))
        .collect();
    let test_cases: Vec<TestCase> = inputs
        .into_iter()
        .zip(outputs.into_iter())
        .map(|(input, output)| TestCase {
            input,
            answer: output,
        })
        .collect();

    Ok(test_cases)
}
