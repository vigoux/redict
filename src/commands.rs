use crate::reply::Reply;
use crate::status::{Status, ReplyKind, Category};
use std::io::{Write, Read, BufReader, BufRead};

pub struct Definition {
    pub source: String,
    pub text: Vec<String>
}

pub fn define<T>(conn: &mut T, database: String, word: String) -> Result<Vec<Definition>, String>
where T: Write + Read {
    writeln!(conn, "DEFINE {} {}", database, word).map_err(|e| e.to_string())?;

    let mut conn = BufReader::new(conn);

    let reply = Reply::from_reader(&mut conn)?;

    // Handle errors separately
    if !reply.status.is_positive() {
        return Err(reply.text);
    }

    // Assert status is 150
    if reply.status != Status(ReplyKind::PositivePreliminary, Category::System, 0) {
        return Err(String::from(format!("Invalid status : {}", reply.status)));
    }

    let mut defs: Vec<Definition> = Vec::new();

    // Now for each 151 answer, get the definition and push it for return
    while let Reply {
        status: Status(ReplyKind::PositivePreliminary, Category::System, 1),

        text: source
    } = Reply::from_reader(&mut conn)? {
        let mut text: Vec<String> = Vec::with_capacity(1);
        // Now get each line until "."
        for l in (&mut conn).lines().filter_map(|l| l.ok()) {
            if l.eq(".") { break; }
            text.push(l);
        }

        // TODO: strip parts of the 151 answer to only keep dictionnary name
        defs.push(Definition { source, text });
    }

    // Our job here is done
    Ok(defs)
}
