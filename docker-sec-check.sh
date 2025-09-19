#!/bin/bash

echo "📋 INICIANDO DIAGNÓSTICO DE SEGURANÇA DO DOCKER..."
echo "==============================================="

# 1. Verifica se o usuário atual está no grupo docker
echo -e "\n🔍 [1] Usuário atual e permissões Docker:"
id | grep docker && echo "✅ Usuário está no grupo docker" || echo "⚠️ Usuário NÃO está no grupo docker"

# 2. Verifica se o daemon está expondo API remota
echo -e "\n🔍 [2] Verificando API do Docker:"
if netstat -tulnp | grep dockerd | grep -q 2375; then
  echo "❌ API remota do Docker (porta 2375) está EXPOSTA!"
else
  echo "✅ API remota do Docker não está exposta (ou está protegida)"
fi

# 3. Lista quem está no grupo docker
echo -e "\n🔍 [3] Membros do grupo docker:"
getent group docker || echo "⚠️ Grupo docker não existe"

# 4. Containers rodando como root
echo -e "\n🔍 [4] Containers rodando como root:"
docker ps --format '{{.Names}}' | while read container; do
  USER=$(docker inspect --format='{{.Config.User}}' "$container")
  if [[ -z "$USER" || "$USER" == "root" ]]; then
    echo "⚠️  $container está rodando como root"
  else
    echo "✅ $container está rodando como $USER"
  fi
done

# 5. Containers com docker.sock montado
echo -e "\n🔍 [5] Containers com /var/run/docker.sock montado:"
docker ps -q | while read cid; do
  docker inspect "$cid" | grep -q "/var/run/docker.sock" && \
  echo "⚠️  $(docker inspect --format '{{.Name}}' "$cid" | cut -c2-) tem docker.sock montado"
done

# 6. Containers com --privileged
echo -e "\n🔍 [6] Containers com --privileged:"
docker ps -q | while read cid; do
  PRIV=$(docker inspect --format='{{.HostConfig.Privileged}}' "$cid")
  if [[ "$PRIV" == "true" ]]; then
    echo "❌ $(docker inspect --format '{{.Name}}' "$cid" | cut -c2-) está com --privileged"
  fi
done

# 7. Verifica uso de imagens oficiais
echo -e "\n🔍 [7] Verificando se imagens são oficiais/verificadas:"
docker images --format '{{.Repository}}:{{.Tag}}' | while read img; do
  if [[ "$img" == *"library/"* || "$img" == *"/"* ]]; then
    echo "✅ $img parece ser confiável"
  else
    echo "⚠️  $img pode não ser oficial/verificada"
  fi
done

# 8. Verifica se firewall está ativo
echo -e "\n🔍 [8] Verificando firewall (ufw):"
ufw status | grep -q "Status: active" && echo "✅ UFW está ativo" || echo "⚠️ UFW está inativo"

# 9. Verifica Portainer (caso esteja rodando)
echo -e "\n🔍 [9] Verificando Portainer:"
if docker ps --format '{{.Names}}' | grep -qi portainer; then
  echo "ℹ️  Portainer está rodando - verifique se está protegido por senha forte e 2FA"
fi

echo -e "\n✅ Diagnóstico concluído. Revise os avisos e recomendações."
