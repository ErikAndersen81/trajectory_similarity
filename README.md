# trajectory_similarity
Calculate the spatiotemporal similarity of two trajectories in GPX format using DISSIM or TRADIS 
## Usage
Clone the repository, navigate to the root directory of the project and build by executing 
> $ cargo build --release
To determine the similarity of all pairs of trajectories in a directory run:
> $ target/release/trajectory_similarity <metric> <directory of gpx-files> <name of csv-file with output>
where metric can be t for TRADIS or d for DISSIM.
