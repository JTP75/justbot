# RustBot

## Brief

A chatbot implemented in rust using LLM APIs. Now equipped with RAG pipeline!

## Features

todo list features

## Installation

todo install instructions

## KanBan

### BACKLOG

- implement chunking
- store multiple files at once (command)
- setup script
    - check for docker, install qdrant
    - install pdftotext
    - create project dirs
    - generate default config files
- add bulk store fn to bot.rs

### READY

- update this file

### IN PROGRESS


### IN REVIEW

- add pipeline output feature
    - needs more testing
- handle pdfs (lopri)

### DONE

- add qdrant docker-compose to startup
- startup
- motd is broken
- fix help command to write aliases
- display loaded conversations
- add history to cli
- collection management commands
- create collection automatically in store function
- rag pipeline
- chat feature
- vector db
- embedding client
- refactoring