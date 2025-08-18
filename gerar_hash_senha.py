# pip3 install argon2-cffi
from argon2 import PasswordHasher
from argon2.exceptions import VerifyMismatchError

# Parâmetros seguros (ajuste conforme seu servidor):
ph = PasswordHasher(
    time_cost=2,        # iterações
    memory_cost=15000,  # 15 MiB
    parallelism=1,      # threads
    hash_len=32,        # 256 bits
    salt_len=16         # 128 bits
)  # usa Argon2id por padrão

def hash_password(password: str) -> str:
    """
    Retorna um hash Argon2id no formato portable ($argon2id$v=19$...).
    """
    return ph.hash(password)

def verify_password(hashed: str, password: str) -> bool:
    """
    Verifica a senha; também indica se é necessário rehash (parâmetros ficaram defasados).
    """
    try:
        ok = ph.verify(hashed, password)
        if ok and ph.check_needs_rehash(hashed):
            # você pode re-hash e atualizar no banco:
            # novo = hash_password(password)
            # salvar novo hash...
            pass
        return True
    except VerifyMismatchError:
        return False

if __name__ == "__main__":
    h = hash_password("MinhaSenhaMuitoForte!123")
    print("hash:", h)
    print("confere:", verify_password(h, "MinhaSenhaMuitoForte!123"))   # True
    print("confere:", verify_password(h, "senha-errada")) 