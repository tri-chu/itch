use git2::{Delta, DiffOptions, Error, IntoCString};
use log::debug;

use crate::{cli::DiffArgs, ctx::Ctx};

fn _diff_command(ctx: &Ctx, args: &DiffArgs) -> Result<(), Error> {
    let base_branch = ctx.repo.find_branch("main", git2::BranchType::Local)?;
    let base_tree = base_branch.into_reference().peel_to_tree()?;
    debug!("{:?}", base_tree);

    let mut options = DiffOptions::new();

    options.include_untracked(true).include_typechange(true);

    let diff_options = Some(&mut options);

    let diff = match &args.target {
        Some(branch) => {
            let target_branch = ctx.repo.find_branch(&branch, git2::BranchType::Local)?;
            let target_tree = target_branch.into_reference().peel_to_tree()?;
            ctx.repo
                .diff_tree_to_tree(Some(&base_tree), Some(&target_tree), diff_options)?
        }
        _ => ctx
            .repo
            .diff_tree_to_workdir(Some(&base_tree), diff_options)?,
    };
    diff.print(git2::DiffFormat::Patch, |_, _, line| {
        let origin = line.origin();
        let color_code = match origin {
            '+' => "\x1b[32m+",
            '-' => "\x1b[31m-",
            _ => "",
        };

        print!(
            "{}{}\x1b[0m",
            color_code,
            line.content().into_c_string().unwrap().to_str().unwrap()
        );
        return true;
    })?;

    diff.deltas().for_each(|delta| match delta.status() {
        Delta::Untracked => {
            println!(
                "\x1b[32mNew file: {}\x1b[0m",
                delta.new_file().path().unwrap().to_str().unwrap()
            );
        }
        _ => {}
    });

    Ok(())
}

pub fn diff_command(ctx: &Ctx, args: &DiffArgs) -> Result<(), ()> {
    _diff_command(&ctx, &args).map_err(|e| {
        println!("Failed to diff: {}", e.message());
    })
}
