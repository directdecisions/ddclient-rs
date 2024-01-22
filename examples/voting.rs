use ddclient_rs::{ApiError, Client};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), ApiError> {
    let client = Client::builder(
        "xapp-1-615817b2fc2700f23eea-c507dt3co5dknpst59c0beqkgm4n26nrr6akqjum".to_string(),
    )
    .api_url("https://api-demo.directdecisions.com/".to_string())
    .build();

    let v = client
        .create_voting(vec![
            "Einstein".to_string(),
            "Maxwell".to_string(),
            "Newton".to_string(),
        ])
        .await?;

    println!("Created voting: {:?}", &v);

    let _ = client
        .vote(
            &v.id,
            "Leonardo",
            HashMap::from([
                ("Einstein".to_string(), 1),
                ("Maxwell".to_string(), 2),
                ("Newton".to_string(), 3),
            ]),
        )
        .await?;

    println!("Leonardo voted for Einstein in voting: {}", &v.id);

    let _ = client
        .vote(
            &v.id,
            "Michelangelo",
            HashMap::from([
                ("Einstein".to_string(), 3),
                ("Maxwell".to_string(), 2),
                ("Newton".to_string(), 1),
            ]),
        )
        .await?;

    println!("Michelangelo voted for Newton in voting: {}", &v.id);

    let choices = client.set_choice(&v.id, "Galileo", 0).await?;
    println!("Appended Galileo to the list of choices: {:?}", choices);

    let results = client.get_voting_results_duels(&v.id).await?;
    println!("Voting results: {:?}", results);

    client.delete_voting(&v.id).await.unwrap();
    println!("Deleted voting with id: {}", &v.id);

    // error handling
    match client.get_voting(&v.id).await {
        Err(ApiError::NotFound) => println!("Voting with id {} not found", &v.id),
        _ => panic!("Expected not found error"),
    }

    Ok(())
}
