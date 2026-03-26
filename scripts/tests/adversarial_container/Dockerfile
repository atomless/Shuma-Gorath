FROM python:3.12-slim

ENV PLAYWRIGHT_BROWSERS_PATH=/ms-playwright

RUN groupadd --system shuma && useradd --system --gid shuma --create-home --home-dir /home/shuma shuma
RUN python3 -m pip install --no-cache-dir "playwright==1.51.1" \
    && python3 -m playwright install --with-deps chromium

WORKDIR /app
COPY scripts/tests/frontier_action_contract.py /app/scripts/tests/frontier_action_contract.py
COPY scripts/tests/frontier_capability_envelope.py /app/scripts/tests/frontier_capability_envelope.py
COPY scripts/tests/adversarial_container/worker.py /app/scripts/tests/adversarial_container/worker.py
COPY scripts/tests/adversarial/lane_contract.v1.json /app/scripts/tests/adversarial/lane_contract.v1.json
COPY scripts/tests/adversarial/sim_tag_contract.v1.json /app/scripts/tests/adversarial/sim_tag_contract.v1.json
COPY scripts/tests/adversarial/frontier_action_contract.v1.json /app/scripts/tests/adversarial/frontier_action_contract.v1.json

RUN chown -R shuma:shuma /app /home/shuma /ms-playwright

USER shuma

ENTRYPOINT ["python3", "-m", "scripts.tests.adversarial_container.worker"]
