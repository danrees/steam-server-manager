-- Your SQL goes here
create table servers (
    id integer primary key not null,
    name text not null unique,
    login text not null,
    install_dir text not null
)