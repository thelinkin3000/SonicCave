-- Add migration script here

create table public.artist
(
    id          uuid default gen_random_uuid() not null
        primary key,
    name        varchar                        not null,
    album_count integer                        not null
);

create table public.album
(
    id         uuid default gen_random_uuid() not null
        primary key,
    name       varchar                        not null,
    year       integer                        not null,
    song_count integer                        not null,
    artist_id  uuid                           not null
        constraint "fk-album-artist_id"
            references public.artist
);

create table public.song
(
    id           uuid    default gen_random_uuid() not null
        primary key,
    title        varchar                           not null,
    path         varchar                           not null,
    genre        varchar                           not null,
    suffix       varchar                           not null,
    content_type varchar                           not null,
    track        integer                           not null,
    duration     integer                           not null,
    album_id     uuid                              not null
        constraint "fk-song-album_id"
            references public.album,
    disc_number  integer default 1                 not null
);

create table public."user"
(
    id       uuid default gen_random_uuid() not null
        primary key,
    username varchar                        not null,
    password varchar                        not null
);

create table public.playlists
(
    id      uuid default gen_random_uuid() not null
        primary key,
    name    varchar                        not null,
    created timestamp                      not null
);

create table public.playlist_items
(
    id          uuid default gen_random_uuid() not null
        primary key,
    playlist_id uuid                           not null
        constraint fk_playlist_items_playlist
            references public.playlists,
    item        integer                        not null,
    song_id     uuid                           not null
        constraint fk_playlist_items_song
            references public.song,
    modified    timestamp                      not null
);
