# multi-machine-dedup

## About ##

multi-machine-dedup is a deduplication tool using SQLite to allow multi-machine features.

multi-machine-dedup is an EDLA project.

The purpose of [edla.org](http://www.edla.org) is to promote the state of the art in various domains.

### Installation ###

```
cargo install multi-machine-dedup
```

## How to use it ##

Index recursively a directory <DIRECTORY_FULL_PATH> labelled with a \<LABEL> in a SQLite database <SQLITE_FILE>
```
 multi-machine-dedup index -l <LABEL> --db <SQLITE_FILE> <DIRECTORY_FULL_PATH>
```

Check a directory
```
 multi-machine-dedup check-integrity -l <LABEL> --db <SQLITE_FILE>
```

Compare two databases
```
 multi-machine-dedup compare --db1 <SQLITE_FILE_1> --db2 <SQLITE_FILE_2>
```

## Example of SQL queries ##

You can use a convenient database tool like [DBeaver CE](https://dbeaver.io) or [SQLiteStudio](https://sqlitestudio.pl) to query the generated SQLite database.

Find top duplicates files larger than <A_SIZE>
```
select label, full_path, hash,size,nb_dup from file , (select hash, count(*) as nb_dup from file where size > <A_SIZE>
group by hash order by nb_dup DESC, size DESC) as T
where file.hash = T.hash  order by nb_dup DESC, size DESC ;
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

* Enable debug mode in PowerShell
```
$Env:LOG='debug';  cargo run ...
```

* Remove LOG environement variable in PorwerShell
```
remove-item Env:LOG
```

* Show help for a \<SUBCOMMAND>
```
multi-machine-dedup <SUBCOMMAND> --help
```
or
```
multi-machine-dedup help <SUBCOMMAND>
```

## Roadmap ##

Inspired by https://github.com/hgrecco/dedup multi-machine-dedup will probably propose similar features.

### License ###
© 2022 Olivier ROLAND. Distributed under the GPLv3 License.
