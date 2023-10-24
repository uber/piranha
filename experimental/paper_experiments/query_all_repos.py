from typing import List
import requests
import argparse


# GitHub API endpoint for repository search
GITHUB_REPO_URL = "https://api.github.com/search/repositories"
# GitHub API endpoint for code search
GITHUB_CODE_SEARCH_URL = "https://api.github.com/search/code"

# # String to search for in the repositories
# search_string = "new SparkConf()"


# parses the command line arguments
def parse_arguments():
    parser = argparse.ArgumentParser(
        description="Searches for a string in the top starred Java and Scala repositories on GitHub"
    )
    parser.add_argument(
        "search_string",
        help="String to search for in the repositories",
    )
    parser.add_argument("token", help="Github CLI token")
    parser.add_argument(
        "--languages",
        nargs="+",
        default=["Java", "Scala"],
        help="List of languages to search for",
    )
    parser.add_argument(
        "--per_page",
        default=100,
        help="Number of results per page (max 100)",
    )
    parser.add_argument(
        "--page",
        default=1,
        help="Page number",
    )
    # argument to specify the path to write the results
    parser.add_argument(
        "--output",
        default="results.csv",
        help="Path to write the results",
    )
    return parser.parse_args()


class MatchedRepo:
    owner: str
    name: str
    stars: int
    search_string: str
    token: str
    languages: list[str]
    number_of_matches: int = 0
    files: list[str] = []

    def __init__(self, name, owner, stars, search_string, token, languages):
        self.name = name
        self.owner = owner
        self.stars = stars
        self.search_string = search_string
        self.token = token
        self.languages = languages
        self.lookup()

    def lookup(self):
        headers = {"Authorization": f"Bearer {self.token}"}
        code_params = {
            "q": f"{self.search_string} repo:{self.owner}/{self.name}",
        }

        code_response = requests.get(
            "https://api.github.com/search/code",
            params=code_params,
            headers=headers,
        )
        if code_response.status_code == 200:
            code_data = code_response.json()
            self.number_of_matches = code_data["total_count"]
            self.files = [
                item["path"]
                for item in code_data["items"]
                if any(l for l in self.languages if l in item["path"])
            ]
    def to_csv(self):
        files = "|".join(self.files)
        return f"{self.name}, {self.owner}, {self.stars}, {self.number_of_matches}, {files}"


def get_repo_info(
    response_json, search_string, token, languages
) -> List[MatchedRepo]:
    repositories = []
    for item in response_json["items"]:
        name = item["name"]
        owner = item["owner"]["login"]
        stars = item["stargazers_count"]

        repositories.append(
            MatchedRepo(name, owner, stars, search_string, token, languages)
        )
    return repositories


def search(token, search_string, languages, output_csv):
    # Set up the headers with your token
    headers = {"Authorization": f"Bearer {token}"}
    # Parameters for the repository search
    _lang_clause = " ".join([f"language:{l}" for l in languages])
    repo_params = {
        "q": f"stars:>100 {_lang_clause}",
        "sort": "stars",
        "order": "desc",
        "per_page": 100,  # Number of results per page (max 100)
        "page": 1,  # Page number
    }

    try:
        # Fetch the top starred repositories
        while True:
            repositories = []
            response = requests.get(
                GITHUB_REPO_URL, params=repo_params, headers=headers
            )

            if response.status_code == 200:
                data = response.json()
                repositories = get_repo_info(
                    response_json=data,
                    search_string=search_string,
                    token=token,
                    languages=languages,
                )
                for r in repositories:
                    if r.number_of_matches > 0:
                        entry = r.to_csv()
                        with open(output_csv, "a+") as f:
                            f.write(entry + "\n")
                            print(entry)
                if "next" not in response.links:
                    break

                repo_params["page"] += 1

            else:
                print(
                    f"Repository request failed with status code {response.status_code}"
                )
                print(response.text)
                break

    except requests.exceptions.RequestException as e:
        print(f"An error occurred: {e}")


args = parse_arguments()
search(
    token=args.token,
    search_string=args.search_string,
    languages=args.languages,
    output_csv=args.output,
)
