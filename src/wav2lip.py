import argparse
import time
from sync import Sync
from sync.common import Audio, GenerationOptions, Video
from sync.core.api_error import ApiError

parser = argparse.ArgumentParser()
parser.add_argument("--video-url", required=True)
parser.add_argument("--audio-url", required=True)
parser.add_argument("--api-key", required=True)
args = parser.parse_args()

client = Sync(
    base_url="https://api.sync.so",
    api_key=args.api_key
).generations

try:
    response = client.create(
        input=[Video(url=args.video_url), Audio(url=args.audio_url)],
        model="lipsync-2",
        options=GenerationOptions(sync_mode="cut_off"),
    )
except ApiError as e:
    print(f"create generation request failed with status code {e.status_code} and error {e.body}")
    exit(1)

job_id = response.id
print(f"Generation submitted successfully, job id: {job_id}")

generation = client.get(job_id)
status = generation.status
while status not in ['COMPLETED', 'FAILED']:
    print(f"polling status for generation {job_id}")
    time.sleep(10)
    generation = client.get(job_id)
    status = generation.status

if status == 'COMPLETED':
    print(f"generation {job_id} completed successfully, output url: {generation.output_url}")
    exit(0)
else:
    print(f"generation {job_id} failed")
    exit(1)
