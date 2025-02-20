# ใช้ official Rust image
FROM rust:1.74

# ตั้งค่าที่ทำงาน
WORKDIR /app

# คัดลอกไฟล์ทั้งหมด
COPY . .

# คอมไพล์โค้ดเป็น binary
RUN cargo build --release

# ใช้ environment variable PORT
CMD ["sh", "-c", "./target/release/blockchain $BLOCKCHAIN_FILE $ACCOUNT_FILE"]
