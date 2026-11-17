//! Apply an explicitly reviewed plan; makes no physical media changes.
use serde::Deserialize;
#[derive(Deserialize)]
struct Plan {
    merges: Vec<[String; 2]>,
    samples: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 4 || args[3] != "--apply" {
        return Err("usage: catalog_cleanup DATA_DIRECTORY PLAN_JSON --apply".into());
    }
    let plan: Plan = serde_json::from_slice(&tokio::fs::read(&args[2]).await?)?;
    if plan.merges.len() > 100 || plan.samples.len() > 100 {
        return Err("cleanup plan exceeds 100 records per category".into());
    }
    let pool = oxevu_db::open(std::path::Path::new(&args[1])).await?;
    for [keep, duplicate] in plan.merges {
        oxevu_library::cleanup::merge_identical(&pool, &keep, &duplicate).await?;
        println!("Merged {duplicate} into {keep}");
    }
    for id in plan.samples {
        oxevu_library::cleanup::exclude_sample(&pool, &id).await?;
        println!("Excluded sample {id}");
    }
    Ok(())
}
