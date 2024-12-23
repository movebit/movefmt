## 1.merge code into develop branch
```
# like this:
git merge fea/optimize_241209
```

## 2.push to remote repo
```
git push origin develop
```

## 3.write ChangeLog.md
### update version
- in Cargo.toml
```
[package]
name = "movefmt"
version = "1.0.7" # update here
```

- src/bin/main.rs
```
fn print_version() {
    println!("movefmt v1.0.7");
}
```


### install new movefmt
```
% cd /the/path/to/your/movefmt
% cargo install --path .
  Installing movefmt v1.0.7...
```

### check movefmt's version
```
edy@MacBookPro movefmt % movefmt --version     
movefmt v1.0.7
# or
edy@MacBookPro movefmt % movefmt -V            
movefmt v1.0.7
edy@MacBookPro movefmt % 
```

### generate recent commit
```
edy@MacBookPro movefmt % cd doc/scripts 
edy@MacBookPro scripts % ls
generate_pretty_commits.sh
edy@MacBookPro scripts % ./generate_pretty_commits.sh 
commits.md generated
edy@MacBookPro scripts % ls
commits.md                      generate_pretty_commits.sh
edy@MacBookPro scripts % 
```

### append change log

## 4.submit code, tag and release version
