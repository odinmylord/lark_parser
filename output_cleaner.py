with open("tmp.txt", "r", encoding="utf-8") as tmp_file:
    lines = tmp_file.readlines()

starting_index = -1
ending_index = len(lines)
for i, line in enumerate(lines):
    if starting_index == -1 and line.strip().startswith("received"):
        starting_index = i + 1

new_lines = lines[starting_index:ending_index]
# remove empty lines
lines = [line.strip() for line in new_lines if line.strip() != ""]
lines_string = "\n".join(lines)
lines_string = lines_string.replace("\nreceived", ", received")

with open("output.txt", "w", encoding="utf-8") as output_file:
    output_file.write(lines_string)