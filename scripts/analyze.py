import anthropic
import time

client = anthropic.Anthropic()

tl_content = open("./data/edappadi_fixed.tl").read()
json_content = open("./data/edappadi.json").read()

prompt = """Analyze this GDELT Global Knowledge Graph record.

Provide:
1. A summary of the news event
2. Key entities (persons, organizations, locations) and their roles
3. Sentiment/tone interpretation — what does the tone score tell us?
4. Notable themes and what they reveal about the event's significance
5. GCAM dimensions — pick 3 interesting scores and explain what they measure
6. How many tokens did this input consume approximately?

Data:
```
{content}
```"""

# --- TeaLeaf format ---
print("=" * 70)
print("TEALEAF FORMAT ANALYSIS")
print("=" * 70)

t0 = time.time()
tl_resp = client.messages.create(
    model="claude-sonnet-4-5-20250929",
    max_tokens=4096,
    messages=[{"role": "user", "content": prompt.format(content=tl_content)}],
)
tl_time = time.time() - t0

print(tl_resp.content[0].text)
print(f"\n--- Stats ---")
print(f"Input tokens:  {tl_resp.usage.input_tokens}")
print(f"Output tokens: {tl_resp.usage.output_tokens}")
print(f"Wall time:     {tl_time:.1f}s")

# --- JSON format ---
print("\n" + "=" * 70)
print("JSON FORMAT ANALYSIS")
print("=" * 70)

t0 = time.time()
json_resp = client.messages.create(
    model="claude-sonnet-4-5-20250929",
    max_tokens=4096,
    messages=[{"role": "user", "content": prompt.format(content=json_content)}],
)
json_time = time.time() - t0

print(json_resp.content[0].text)
print(f"\n--- Stats ---")
print(f"Input tokens:  {json_resp.usage.input_tokens}")
print(f"Output tokens: {json_resp.usage.output_tokens}")
print(f"Wall time:     {json_time:.1f}s")

# --- Comparison ---
print("\n" + "=" * 70)
print("COMPARISON")
print("=" * 70)
tl_in = tl_resp.usage.input_tokens
json_in = json_resp.usage.input_tokens
savings = (1 - tl_in / json_in) * 100
print(f"TeaLeaf input tokens: {tl_in}")
print(f"JSON input tokens:    {json_in}")
print(f"Token savings:        {savings:.1f}%")
print(f"TeaLeaf wall time:    {tl_time:.1f}s")
print(f"JSON wall time:       {json_time:.1f}s")
