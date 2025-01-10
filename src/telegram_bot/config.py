COMMANDS = [
    ("werhatdiehandandermaus", "wer ist das wohl?"),
    ("add", "<imdb url> adds movie to queue by given imdb url"),
    ("delete", "<title> deletes title from database"),
    ("watch", "<title> mark title as watched"),
    ("queue", "retrieves complete queue"),
    (
        "wostream",
        "<title> searches streaming providers for this title(title needs to be in this project)",
    ),
]

NAME = "timhatdiehandandermaus"
DESCRIPTION = (
    "interact with the timhatdiehandandermaus API (https://api.timhatdiehandandermaus.consulting)"
)
SHORT_DESCRIPTION = "interact with tim"
