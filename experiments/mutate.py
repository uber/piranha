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
    import requests
    import json

    GITHUB_TOKEN = "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"
    HEADERS = {
        "Authorization": f"Bearer {GITHUB_TOKEN}"
    }

    # Construct the GraphQL query
    query = """
    query {
        search(query: "language:java stars:>1 NOT leetcode NOT book NOT algorithms NOT examples NOT tutorial", type: REPOSITORY, first: 10) {
            edges {
                node {
                    ... on Repository {
                        name
                        owner {
                            login
                        }
                        url
                        stargazers {
                            totalCount
                        }
                    }
                }
            }
        }
    }
    """

    request = requests.post('https://api.github.com/graphql', json={'query': query}, headers=HEADERS)

    if request.status_code == 200:
        result = request.json()
        # Extract the repository information from the GraphQL response
        repos = result['data']['search']['edges']
        return [{"name": repo['node']['name'],
                 "owner": repo['node']['owner']['login'],
                 "url": repo['node']['url'],
                 "stars": repo['node']['stargazers']['totalCount']} for repo in repos]
    else:
        print(f"Query failed with status code {request.status_code}")
        print(request.text)
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
