# Pagerank Convergence Optimization with NCD Matrices

This project investigates the convergence time of the stationary distribution of PageRank as a function of the damping factor α, using Nearly Completely Decomposable (NCD) matrices derived from web graphs.

## Objective

The goal is to study how the convergence of PageRank’s stationary distribution is affected when the transition matrix is transformed into an NCD matrix. An NCD matrix is a Markov chain whose state space can be partitioned such that transitions within the same group are much more frequent than transitions between different groups.

The standard PageRank matrix is defined as:

$G = \alpha P + \alpha (1/N) f e^t + (1- \alpha)(1/N) e e^t$

where:
- $P$ is the $N \times N$ Markov chain matrix modeling web page relevance.
- $f$ is a column vector representing dangling rows in $P$.
- $e$ is a column vector of ones.
- $\alpha$ is the probability that the random surfer follows a link ($\alpha$ = 0.85 in the original PageRank).

The stationary distribution $\pi$ satisfies $\pi G = \pi$ and $\pi e = 1$.

## NCD Matrix Construction

Two different methods were implemented to obtain NCD matrices from a given web graph:

### 1. Random Arc Deletion

Arcs are randomly removed from the original graph in successive steps (e.g., 10%, then an additional 10%, etc.). This naive approach does not guarantee a proper partition and can sometimes increase the convergence time, especially when arcs from low-outdegree nodes are removed. However, it can also isolate strongly connected components and thus create an NCD structure that accelerates convergence.

### 2. Random Group Generation

A random partition of the graph is created first. Then, arcs between different groups are removed with a probability given by a sigmoid function: $1 + e^{-(8x-3)}$ where $x$ is the weight of the arc. This function is chosen so that arcs with a weight greater than 0.5 are very likely to be deleted. This method explicitly exploits the NCD structure by enforcing a partition of the state space.

## Rust Implementation

The entire simulation suite is implemented in Rust for performance and safety. Key features include:

- **CSC Matrix Storage** – The transition matrices are stored in Compressed Sparse Column (CSC) format to minimize memory footprint and enable efficient arithmetic operations, which is critical for large web graphs with millions of nodes.
- **Multi‑threaded Computations** – Stationary distribution and convergence metrics are computed in parallel using a Thread Pool implemented with native threads, significantly reducing simulation time for multiple $\alpha$ values and multiple graph variants.
- **Memory Optimisations** – Custom iterators and zero‑copy abstractions are used to traverse sparse structures without unnecessary allocations. The CSC format also enables fast column‑wise operations, which are essential for the power iteration method.

## Getting Started

Clone the repository :
```bash
git clone https://github.com/LugolBis/Ranking.git
cd Ranking
```

Create and complete the `config.json` file :
```bash
echo "{}" > config.json 
```
Run the project :
```bash
cargo run --release -p ranking --help
```
