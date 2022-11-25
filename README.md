# LAN IM Server

![build](https://github.com/Tramis/simple-im/actions/workflows/rust.yml/badge.svg)

just for practice

not only for LAN in theory, it's just my use scene

## Usage

```bash
.
├── chatroom
├── b
│   └── statics
│       ├── b.css
│       └── a.js
└── a
    └── index.html

# for directory tree as above
user@user:path/to/bin$ ./chatroom -d b/statics -i a/index.html
```

## Update

- add parameter to specify index.html and static resouces (`clap`)
- add compiling-config log (`log4rs`)

## TODO

- support: records
- ~~highly performance with rust async runtime, such as tokio~~
- support: upload files
- support: image
- support: rich-text
