FROM python:3.12-slim

RUN groupadd --system shuma && useradd --system --gid shuma --create-home --home-dir /home/shuma shuma

WORKDIR /app
COPY scripts/tests/adversarial_container/worker.py /app/worker.py

USER shuma

ENTRYPOINT ["python3", "/app/worker.py"]
