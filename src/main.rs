use rand::seq::SliceRandom;
use std::collections::HashSet;
use std::io::{self, Read};

#[derive(Clone)]
struct Contestant {
    wins: i32,
    draws: i32,
    losses: i32,
    history: HashSet<String>,
    name: String,
}

fn main() {
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .expect("Could not read from stdin.");
    let mut line_iter = buffer.split('\n');

    let mut contestants: Vec<Contestant> = Vec::new();
    for csv_name in line_iter.next().unwrap().split(',').map(|x| x.trim()) {
        contestants.push(Contestant {
            wins: 0,
            draws: 0,
            losses: 0,
            history: HashSet::new(),
            name: csv_name.to_string(),
        });
    }
    contestants.shuffle(&mut rand::thread_rng());

    for result in line_iter.filter(|x| x.len() > 0) {
        let csvs: Vec<&str> = result.split(',').collect();

        if csvs.len() == 4 {
            let mut con = contestants
                .iter_mut()
                .filter(|x| x.name == csvs[0])
                .nth(0)
                .unwrap();
            match csvs[1] {
                "W" => con.wins += 1,
                "D" => con.draws += 1,
                "L" => con.losses += 1,
                _ => panic!("Unrecognized game result: {}", csvs[1]),
            }
            con.history.insert(csvs[2].to_string());

            let mut con = contestants
                .iter_mut()
                .filter(|x| x.name == csvs[2])
                .nth(0)
                .unwrap();
            match csvs[3] {
                "W" => con.wins += 1,
                "D" => con.draws += 1,
                "L" => con.losses += 1,
                _ => panic!("Unrecognized game result: {}", csvs[3]),
            }
            con.history.insert(csvs[0].to_string());
        } else if csvs.len() == 2 {
            let mut con = contestants
                .iter_mut()
                .filter(|x| x.name == csvs[0])
                .nth(0)
                .unwrap();
            con.wins += 1;
        }
    }

    // Only generate new games for the players with fewest played games. This finishes/starts the
    // current round
    let fewest_games = contestants
        .iter()
        .map(|x| x.wins + x.draws + x.losses)
        .min()
        .unwrap();
    let mut matchups: Vec<(String, String)> = Vec::new();
    let mut available_cons: Vec<Contestant> = contestants
        .iter()
        .filter(|x| x.wins + x.draws + x.losses == fewest_games)
        .map(|x| x.clone())
        .collect();

    while available_cons.len() > 1 {
        // We will find a match pairing for the player with the least options amongst the available
        // player pool.
        let a_cons_set = available_cons
            .iter()
            .map(|x| x.name.to_string())
            .collect::<HashSet<String>>();
        let selected_con_index = available_cons
            .iter()
            .enumerate()
            .min_by_key(|x| a_cons_set.difference(&x.1.history).count())
            .unwrap()
            .0;
        let selected_con = available_cons.swap_remove(selected_con_index);

        // Our match will be a player we have not played with the closest score to our own
        let matched_con_index = available_cons
            .iter()
            .enumerate()
            .filter(|x| !selected_con.history.contains(&x.1.name))
            .min_by_key(|x| {
                ((2 * selected_con.wins + selected_con.draws) - (2 * x.1.wins + x.1.draws)).abs()
            })
            .unwrap()
            .0;
        let matched_con = available_cons.swap_remove(matched_con_index);

        matchups.push((selected_con.name.to_string(), matched_con.name.to_string()));
    }

    print_output(contestants, matchups, available_cons);
}

fn print_output(
    contestants: Vec<Contestant>,
    matchups: Vec<(String, String)>,
    available_cons: Vec<Contestant>,
) {
    println!(
        r#"<!DOCTYPE html>
<html>
  <head>
    <title>Swiss-System Tournament Results</title>
  </head>
  <body>
    <h1>Swiss-System Tournament Results</h1>"#
    );

    print_results(contestants);
    print_matchups(matchups, available_cons);

    println!("  </body>\n</html>");
}

fn print_results(mut contestants: Vec<Contestant>) {
    println!(
        r#"    <table>
      <thead>
        <tr>
          <th>Player Name</th>
          <th>Score</th>
          <th>Games Played</th>
          <th>Wins</th>
          <th>Draws</th>
          <th>Losses</th>
        </tr>
      </thead>
      <tbody>"#
    );

    contestants.sort_unstable_by(|a, b| {
        (2 * b.wins + b.draws)
            .cmp(&(2 * a.wins + a.draws))
            .then_with(|| (a.wins + a.draws + a.losses).cmp(&(b.wins + b.draws + b.losses)))
    });
    for c in contestants.iter() {
        println!(
            r#"        <tr>
          <td>{}</td>
          <td>{:.1}</td>
          <td>{}</td>
          <td>{}</td>
          <td>{}</td>
          <td>{}</td>
        </tr>"#,
            c.name,
            ((2 * c.wins + c.draws) as f32) / 2.0,
            c.wins + c.draws + c.losses,
            c.wins,
            c.draws,
            c.losses
        );
    }

    println!("      </tbody>\n    </table>");
}

fn print_matchups(matchups: Vec<(String, String)>, available_cons: Vec<Contestant>) {
    println!("    <h2>Next Round Matchups</h2>\n    <ul>");

    for m in matchups.iter() {
        println!("      <li>{} vs. {}</li>", m.0, m.1);
    }

    for c in available_cons.iter() {
        println!("      <li>{} gets a BYE</li>", c.name);
    }

    println!("    </ul>");
}
