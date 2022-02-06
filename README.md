# multi-machine-dedup

## About ##

multi-machine-dedup is a deduplication tool using SQLite to allow multi-machine features.

multi-machine-dedup is an EDLA project.

The purpose of [edla.org](http://www.edla.org) is to promote the state of the art in various domains.

## How to use it ##

Index recursively a directory <DIRECTORY_FULL_PATH> labelled with a \<LABEL> in a SQLite database <SQLITE_FILE>
```
 cargo run -- index -l <LABEL> --db <SQLITE_FILE> <DIRECTORY_FULL_PATH>
```

Check a directory
```
 cargo run -- check-integrity -l <LABEL> --db <SQLITE_FILE>
```

Compare two databases
```
 cargo run -- compare --db1 <SQLITE_FILE_1> --db2 <SQLITE_FILE_2>
```

## Example of SQL queries ##

You can use a convenient database tool like [DBeaver CE](https://dbeaver.io) or [SQLiteStudio](https://sqlitestudio.pl) to query the generated SQLite database.

Find top duplicates files larger than <A_SIZE>
```
select *, count(*) as nb_dup from file where size > <A_SIZE> group by hash order by nb_dup DESC, size DESC ;
```

Find all files with the same <CRC_VALUE>
```
select * from file where hash=<A_CRC_VALUE> ;
```

Find all files with image/jpeg MIME-type.
```
select * from hash where mime like "image/jpeg" ;
```

## Tips ##

Enable debug mode in PowerShell
```
$Env:RUST_LOG='debug';  cargo run ...
```

Show help of a \<SUBCOMMAND>
```
cargo run -- <SUBCOMMAND> --help
```

## Roadmap ##

Inspired by https://github.com/hgrecco/dedup multi-machine-dedup will probably propose similar features.

### License ###
© 2022 Olivier ROLAND. Distributed under the GPLv3 License.
