# Sparse Matrix Ranking in Rust

A lightweight Rust project implementing ranking algorithms using **sparse matrix representations**.  
The goal is to efficiently compute rankings on large, mostly empty datasets such as graphs, recommendation systems, or link analysis problems.

## Overview

This project explores how **sparse matrices** can be leveraged to perform scalable ranking computations while minimizing memory usage and improving performance.

Key ideas:
- Represent large datasets using **sparse matrix structures**
- Perform iterative ranking computations on sparse data
- Optimize for **performance and memory efficiency** using Rust

## Features

- Sparse matrix-based data representation
- Efficient ranking computation
- Memory-efficient handling of large datasets
- Written in thread-safe **Rust**

## Tech Stack

- **Rust**
- Sparse matrix structures
- Iterative ranking algorithms

## Getting Started

Clone the repository :
```bash
git clone https://github.com/LugolBis/Ranking.git
cd Ranking
```

Create and complete the `.env` file :
```bash
echo "MATRIX_PATH=''" > .env
```
Run the project :
```bash
cargo run --release
```
