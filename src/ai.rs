use super::board::Board;
use super::board::Point;
use super::board::Content;
use super::interaction::ActionType;


pub fn generate_move(board: &Board) -> Vec<ActionType>{
    let safe_flags = known_safe_flags(board);
    if safe_flags.len() > 0{
        return safe_flags.iter().map(|point| ActionType::Flag(point.clone())).collect()
    }

    let safe_clicks = known_safe_clicks(board);
    if safe_clicks.len() > 0{
        return safe_clicks.iter().map(|point| ActionType::Click(point.clone())).collect()
    }
    let probabilities = get_naive_mine_probabilities(board);
    match safest_click(probabilities){
        None => vec![],
        Some((point, proba)) => {
            println!("Naively estimating probability at {}", proba);
            vec![ActionType::Click(point)]
        }
    }
}

fn known_safe_flags(board: &Board) -> Vec<Point> {
    board.size.points().iter()
         .filter(|point| !board.retrieve_cell(point).flagged)
         .filter(|point| !board.retrieve_cell(point).known)
         .map(|point| (point.clone(), get_naive_mine_probability(board, &point, true)))
         .filter(|(_, proba)| *proba == 1.0)
         .map(|(point, _)| point.clone())
         .collect()
}

fn known_safe_clicks(board: &Board) -> Vec<Point> {
    board.size.points().iter()
         .filter(|point| !board.retrieve_cell(point).flagged)
         .filter(|point| !board.retrieve_cell(point).known)
         .map(|point| (point.clone(), get_naive_mine_probability(board, &point, false)))
         .filter(|(_, proba)| *proba == 0.0)
         .map(|(point, _)| point.clone())
         .collect()
}

fn safest_click(point_probabilities: Vec<(Point, f32)>) -> Option<(Point, f32)>{
    point_probabilities.iter()
        .filter(|(_, proba)| *proba > 0.0 && *proba < 1.0 )
        .fold(None, |acc, (point, proba)| { //FIXME: painfully similar to the code in naive_mine_probability
            match acc {
                None => Some((point.clone(), *proba)),
                Some(acc) => {
                    let acc_proba = acc.1;
                    let result = {
                        if *proba == 0.0 {
                            (point.clone(), *proba)
                        } else {
                            if *proba < acc_proba {(point.clone(), *proba)} else {acc} //TODO: why do i need all these derefs
                            // oh i think it's iter vs into_iter
                        }
                    };
                    Some(result)
                }
            }
        })
}

fn get_naive_mine_probability(board: &Board, point: &Point, pessimistic: bool) -> f32 {
    let cell = board.retrieve_cell(point);
    if cell.known{
        return match cell.content {
            Content::Mine => 1.0,
            Content::Empty => 0.0
        }
    }
    if cell.flagged {
        return 1.0
    }
    let probability = board.neighbor_points(point).iter()
         .map(|point| (point, board.retrieve_cell(point)))
         .filter(|(_, neighbor)| neighbor.known)
         .map(|(point, neighbor)| {
             let flagged = board.count_flagged_neighbors(point);
             let mined = board.count_assumed_mined_neighbors(point);
             let unknown = board.count_unknown_neighbors(point);
             (neighbor.neighbors - mined) as f32/(unknown - flagged) as f32
         })
         .fold(None, |acc, proba| {
             match acc {
                 None => Some(proba),
                 Some(acc) => {
                     let result = {
                         if proba == (pessimistic as usize) as f32 { //hmmmm
                             proba
                         } else {
                             if pessimistic {
                                 if proba > acc {proba} else {acc}
                             }
                             else {
                                 if proba < acc {proba} else {acc}
                             }
                         }
                     };
                     Some(result)
                 }
             }
         });
    match probability {
        Some(p) => p,
        None => (board.mine_count as f32) / (board.size.area() as f32)
    }
}

fn get_naive_mine_probabilities(board: &Board) -> Vec<(Point, f32)>{
    board.size.points().iter()
        .map(|point| (point.clone(), get_naive_mine_probability(board, &point,false)))
        .collect()
}
