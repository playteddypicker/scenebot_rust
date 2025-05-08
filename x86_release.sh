docker build -f Dockerfile.build -t rust-x86-build:latest .

container_id=$(docker create --platform=linux/amd64 rust-x86-build:latest)
docker cp "$container_id":/app/target/release/scene_rust ./scene_rust
docker rm "$container_id"

echo "✅ scene_rust 바이너리 추출 완료!"
file ./scene_rust
