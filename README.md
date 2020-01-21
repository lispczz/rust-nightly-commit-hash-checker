## Rust Nightly Commit Hash Checker 
Usually we expect some new features or fixes to be included into the rust nightly build.   
If you already know the corresponding git commit hash, you can use this tool to check whether this commit has been included into the latest nightly build.

## Example
You should set some environment variables before running this tool, including `USER_TOKEN` and `USER_NAME` of your github account.  
These environment variables are required for the Github GraphQL API.  
You can also set them [in the `.env` file](https://docs.rs/dotenv/0.15.0/dotenv/). 
 
```bash
 $ cargo run -- bff216c56f472dd751d3da636027d5e2d821e979
    Finished dev [unoptimized + debuginfo] target(s) in 0.18s
     Running `target/debug/rust-nightly-commit-hash-checker bff216c56f472dd751d3da636027d5e2d821e979`
nightly:
version: 1.42.0-nightly (b5a3341f1 2020-01-20)
hash: b5a3341f1b8b475990e9d1b071b88d3c280936b4
Not found! Nightly build doesn't contain this commit: bff216c56f472dd751d3da636027d5e2d821e979
```

## Notes
This program queries the Github GraphQL API for 5 times. Each time it will fetch 100 ancestor commits of the nightly build. So a total of 500 commits will be checked.      
You can adjust the `trail_count` from the default value of `5` to other values. 

