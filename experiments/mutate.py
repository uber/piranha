import subprocess
import requests
import os
import random
from tree_sitter_languages import get_parser


class Mutator:
    def __init__(self, mutation_probability):
        self.mutation_probability = mutation_probability
        self.parser = get_parser("java")

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
        mutated_code = self._mutate_if_statements(code)
        return mutated_code

    def mutation_strategy_2(self, code):
        # Placeholder implementation
        return code

    def mutation_strategy_3(self, code):
        # Placeholder implementation
        return code

    def mutation_strategy_4(self, code):
        # Placeholder implementation
        return code

    def _mutate_if_statements(self, code):
        for _ in range(5):
            code_bytes = bytes(code, 'utf8')
            tree = self.parser.parse(code_bytes)
            cursor = tree.walk()

            nodes = [cursor.node]  # Start with the root node
            while nodes:
                node = nodes.pop()  # Pop a node from the stack
                if node.type == 'if_statement':
                    # Identify start and end byte of the condition within the if statement
                    condition_start = node.children[1].start_byte
                    condition_end = node.children[1].end_byte

                    # Negate the condition
                    new_condition_bytes = b"(hello)"
                    mutated_code_bytes = code_bytes[:condition_start] + new_condition_bytes + code_bytes[condition_end:]

                    code = mutated_code_bytes.decode('utf8')
                    break  # Exit the loop once a mutation is made

                    # Add children of the current node to the stack for further exploration
                nodes.extend(node.children)

        return code

    def process_files(self, root_dir):
        for subdir, _, files in os.walk(root_dir):
            for file in files:
                if file.endswith(".java"):
                    file_path = os.path.join(subdir, file)
                    if self.should_mutate(file_path):
                        self.mutate(file_path)


def get_top_repos():
    GITHUB_TOKEN = "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"  # Replace this with your new token
    HEADERS = {
        "Authorization": f"Bearer {GITHUB_TOKEN}"
    }

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
        # Construct the clone URL from owner and name
        clone_url = f"https://github.com/{repo['owner']}/{repo['name']}.git"
        subprocess.run(['git', 'clone', clone_url, f"./repos/{repo['name']}"])


def main():
    #repos = get_top_repos()
    #clone_repos(repos)
    mutator = Mutator(mutation_probability=0.2)
    mutator.process_files("./repos")


if __name__ == "__main__":
    main()
