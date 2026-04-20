pub fn run(dry_run: bool, exclude_stale: bool) -> anyhow::Result<()> {
    let current = std::env::current_dir()?;
    let written = crate::compile::compile_repo(&current, dry_run, exclude_stale)?;

    for path in written {
        println!("wrote {}", path.display());
    }

    Ok(())
}
