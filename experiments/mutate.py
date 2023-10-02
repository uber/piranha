import subprocess
import requests
import os
import random


class Mutator:
    def __init__(self, mutation_probability):
        self.mutation_probability = mutation_probability

    def should_mutate(self, file_path):
        # This does not need to be a bernoulli distribution
        return random.random() < self.mutation_probability

    def mutate(self, file_path):
        with open(file_path, 'r') as f:
            code = f.read()

        mutation_function = random.choice([
            self.mutation_strategy_1,
            self.mutation_strategy_2,
            self.mutation_strategy_3,
            self.mutation_strategy_4,
        ])
        mutated_code = mutation_function(code)

        with open(file_path, 'w') as f:
            f.write(mutated_code)

    def mutation_strategy_1(self, code):
        # Replace this with the actual mutation logic
        return code  # Placeholder implementation

    def mutation_strategy_2(self, code):
        # Placeholder implementation
        return code

    def mutation_strategy_3(self, code):
        # Placeholder implementation
        return code

    def mutation_strategy_4(self, code):
        # Placeholder implementation
        return code

    def process_files(self, root_dir):
        for subdir, _, files in os.walk(root_dir):
            for file in files:
                if file.endswith(".java"):
                    file_path = os.path.join(subdir, file)
                    if self.should_mutate(file_path):
                        self.mutate(file_path)



def get_top_repos():
    GITHUB_API_URL = "https://api.github.com/search/repositories"
    GITHUB_TOKEN = "your_github_token"
    QUERY = "language:java"
    SORT = "stars"
    ORDER = "desc"
    PER_PAGE = 20

    headers = {
        "Authorization": f"token {GITHUB_TOKEN}",
        "Accept": "application/vnd.github.v3+json"
    }

    params = {
        "q": QUERY,
        "sort": SORT,
        "order": ORDER,
        "per_page": PER_PAGE
    }

    response = requests.get(GITHUB_API_URL, headers=headers, params=params)

    if response.status_code == 200:
        items = response.json()['items']
        return [{"name": item['name'], "clone_url": item['clone_url']} for item in items]
    else:
        print(f"Error fetching data from GitHub API: {response.status_code}")
        print(response.json())
        return []

def clone_repos(repos):
    for repo in repos:
        subprocess.run(['git', 'clone', repo['clone_url'], f"./repos/{repo['name']}"])

def main():
    repos = get_top_repos()
    clone_repos(repos)
    mutator = Mutator(mutation_probability=0.2)
    mutator.process_files("./repos")


if __name__ == "__main__":
    main()
