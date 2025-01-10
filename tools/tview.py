"""
tview.py
8/3/2024

Author: Jude Hoogterp
Contact: jude@highland.software

Description:
This script reads UTF-8 data from all files in a specified project directory,
excluding specified files and directories, and concatenates their content
into a single text file.

License: MIT License (see LICENSE file)
"""

import os
import argparse
from tqdm import tqdm

# Function to read files and append to the overview
def append_file_contents(file_path, overview):
    try:
        with open(file_path, 'r', encoding='utf-8', errors='ignore') as file:
            content = file.read()
        # Filter out non-printable characters
        content = ''.join(filter(lambda x: x.isprintable() or x.isspace(), content))
        overview.write(f"\n{file_path}:\n")
        overview.write(content)
    except Exception as e:
        print(f"Skipping file {file_path} due to an error: {e}")

def should_exclude(path, exclude_list):
    return any(exclude in path for exclude in exclude_list)

def load_exclude_list(viewignore_file):
    exclude_list = []
    try:
        with open(viewignore_file, 'r', encoding='utf-8') as f:
            exclude_list = [line.strip() for line in f.readlines() if line.strip() and not line.startswith('#')]
    except Exception as e:
        print(f"Error reading {viewignore_file}: {e}")
    return exclude_list

def find_project_root(start_path, project_name):
    current_path = start_path
    while True:
        if os.path.basename(current_path) == project_name:
            return current_path
        parent_path = os.path.dirname(current_path)
        if parent_path == current_path:
            raise FileNotFoundError(f"Project root '{project_name}' not found from {start_path}")
        current_path = parent_path

def main(exclude_list, output_file, project_name):
    # Find the project root directory
    project_root = find_project_root(os.getcwd(), project_name)
    
    # Get all file paths in the project directory and subdirectories, excluding specified files and directories
    file_paths = [os.path.join(root, file) 
                  for root, dirs, files in os.walk(project_root) 
                  for file in files
                  if not should_exclude(root, exclude_list) and not should_exclude(file, exclude_list)]

    # Open the output file once and write all contents
    with open(output_file, 'a', encoding='utf-8') as overview:
        for file_path in tqdm(file_paths, desc="Processing files"):
            append_file_contents(file_path, overview)

if __name__ == "__main__":
    # Argument parser
    parser = argparse.ArgumentParser(description="Concatenate files into an overview")
    parser.add_argument('--exclude', '-x', type=str, nargs='+', help="Files or directories to exclude", default=[])
    parser.add_argument('--output', '-o', type=str, help="Output file name", default="tview.txt")
    parser.add_argument('--viewignore', '-v', type=str, help="Path to .viewignore file", default="./tools/.viewignore")
    parser.add_argument('--project', '-p', type=str, help="Name of the project root folder", required=True)
    args = parser.parse_args()

    # Load exclusions from .viewignore file
    exclude_list = load_exclude_list(args.viewignore)
    exclude_list.extend(args.exclude)

    # Clear the output file
    with open(args.output, 'w', encoding='utf-8') as f:
        pass

    main(exclude_list, args.output, args.project)
