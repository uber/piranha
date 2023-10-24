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
    return parser.parse_args()


def lookup_keyword_in_repos(headers, search_string, repos):
    for repo in repos:
        code_params = {
            "q": f"{search_string} repo:{repo['owner']}/{repo['name']}",
        }

        code_response = requests.get(
            "https://api.github.com/search/code",
            params=code_params,
            headers=headers,
        )

        if code_response.status_code == 200:
            code_data = code_response.json()
            if code_data["total_count"] > 0:
                print(f"{repo['owner']}/{repo['name']} - {code_data['total_count']}")


def get_repo_info(response_json):
    repositories = []
    for item in response_json["items"]:
        repositories.append(
            {"name": item["name"], "owner": item["owner"]["login"]}
        )
    return repositories


def search(token, search_string, languages):
    # Set up the headers with your token
    headers = {"Authorization": f"Bearer {token}"}
    # Parameters for the repository search
    _lang_clause= " ".join([f"language:{l}" for l in languages])
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
                repositories = get_repo_info(data)
                
                if "next" not in response.links:
                    break
                
                repo_params["page"] += 1

            else:
                print(
                    f"Repository request failed with status code {response.status_code}"
                )
                print(response.text)
                break
            
            lookup_keyword_in_repos(headers, search_string, repositories)

    except requests.exceptions.RequestException as e:
        print(f"An error occurred: {e}")



args = parse_arguments()
search(args.token, args.search_string, args.languages)
