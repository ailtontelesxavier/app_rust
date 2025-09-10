

document.getElementById('btnCancelar').addEventListener('click', function () {
    window.location.href = "/externo/contato";
});

document.addEventListener('DOMContentLoaded', function () {

    // Função para lidar com o evento de submit
    document.getElementById("btnSubmit").addEventListener("click", async function () {

        const form = document.getElementById('form_contato');

        // Obtendo os dados do formulário
        const formData = new FormData(form);

        // Convertendo FormData para um objeto simples
        const data = Object.fromEntries(formData.entries());
        console.log("data:", data);


        // {#pega todos os items#}
        const container = document.getElementById("item-container");
        const inputs = container.querySelectorAll("input");

        const itens = {};
        inputs.forEach(input => {
            if (input.value.trim() !== '') {
                itens[input.name] = input.value;
            }
        });

        data.itens = Object.keys(itens).length > 0 ? JSON.stringify(itens) : JSON.stringify([]);


        if (!data.cidade_id || data.cidade_id.trim() === "") {
            showMessage("Por favor, selecione a cidade novamente.");
            return;
        }

        /* if (!form.checkValidity()) {
            const invalidFields = Array.from(form.elements).filter(
                (el) => el instanceof HTMLInputElement || el instanceof HTMLSelectElement || el instanceof HTMLTextAreaElement
            ).filter((el) => !el.validity.valid);

            const fieldNames = invalidFields.map((field) => field.name).filter(Boolean);

            showMessage(`Por favor, preencha os seguintes campos obrigatórios: ${fieldNames.join(", ")}`);
            return;
        } */

        showLoader();
        //form.submit();

        let url = "/externo/contato-form-pronaf";
        let method = "post";
        let Content_Type = 'multipart/form-data';
        // Envia os dados do formulário usando Axios
        await axios.post(url, data, { headers: { "Content-Type": Content_Type }, })
            .then(function (response) {
                hideLoader();
                showMessage("Formulario enviado com sucesso!");

                setTimeout(() => {
                    window.location.href = `/externo/confirmacao-cadastro-basa?protocolo=${response?.data?.protocolo}&nome=${response?.data?.nome}&cpf=${response?.data?.cpf}`
                }, 3000);

            })
            .catch(function (error) {
                hideLoader();
                //console.error("Erro inesperado:", error.response?.data?.detail);
                //showMessage("Erro ao processar requisição: " +error.response?.data?.detail);
                var msg = '';
                if (axios.isAxiosError(error)) {
                    const detail = error.response?.data?.detail;
                    if (Array.isArray(detail)) {
                        // Caso seja um array de erros
                        detail.forEach((d) => {
                            msg += (d?.msg || JSON.stringify(d)) + '\n';
                        });
                    } else if (typeof detail === "object" && detail !== null) {
                        // Caso seja objeto {campo: [erros]}
                        Object.entries(detail).forEach(([field, errors]) => {
                            if (Array.isArray(errors)) {
                                errors.forEach((err) => {
                                    msg += `${field}: ${err.message}\n`;
                                });
                            }
                        });
                    } else {
                        // Caso seja uma string simples
                        msg += detail + '\n';
                    }

                    console.log("Erros capturados:", msg);
                    showMessage("Erro ao processar requisição:\n" + msg);
                } else {
                    console.error("Erro inesperado:", error);
                }
            });
    });

});

