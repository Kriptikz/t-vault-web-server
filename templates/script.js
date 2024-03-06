// decode base64 encoded transactions to Uint8Array for wallet to sign
function decodeTx(encodedTx) {
  let binaryString = atob(encodedTx);

  let byteArray = new Uint8Array(binaryString.length);
  for (let i = 0; i < binaryString.length; i++) {
    byteArray[i] = binaryString.charCodeAt(i);
  }

  return byteArray;
}

// encode a UInt8Array serialized tx into a base64 encoded string
function encodeTx(byteArray) {
  let binaryString = "";
  for (let i = 0; i < byteArray.byteLength; i++) {
    binaryString += String.fromCharCode(byteArray[i]);
  }
  
  let encodedTx = btoa(binaryString);
  return encodedTx;
}

// used by hx-vals to add the pubkey to requests
function getPubkey() {
  const wallet = window.solflare;
  if (wallet && wallet.isConnected) {
    return wallet.publicKey.toString();
  }
  return "";
}

// used by tx-modal signAndSend button
async function signAndSend(element) {
  const wallet = window.solflare;
  if (!wallet.isConnected) {
    console.log("Wallet is not connected.");
    return;
  }
  try {
    let encodedTx = element.getAttribute("encoded-tx");
    let txId = element.getAttribute("tx-id");

    let tx = solanaWeb3.Transaction.from(decodeTx(encodedTx));

    const signedTransaction = await wallet.signTransaction(tx);

    let data = {
      txId,
      encodedSerializedTx: encodeTx(signedTransaction.serialize())
    }

    htmx.ajax('POST', '/tx-submit', {
      target: '.modal-content',
      swap: 'innerHTML',
      values: data
    });
  } catch (err) {
    console.error("Failed to process transaction:", err);
  }

}

document.addEventListener('DOMContentLoaded', () => {
  const wallet = window.solflare;

  if (!wallet) {
    console.log("Solana wallet extension not found.");
    return;
  }



  async function handleTransaction(endpoint) {
    if (!wallet.isConnected) {
      console.log("Wallet is not connected.");
      return;
    }

    try {
      const response = await fetch(endpoint, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ public_key: wallet.publicKey.toString() }),
      });

      if (!response.ok) {
        throw new Error('Network response was not ok');
      }

      const { encodedTx } = await response.json();

      let tx = solanaWeb3.Transaction.from(decodeTx(encodedTx));

      const signedTransaction = await wallet.signAndSendTransaction(tx);
      console.log("Transaction signature:", signedTransaction.signature);
    } catch (err) {
      console.error("Failed to process transaction:", err);
    }
  }

  function updateUI() {
    const connectButton = document.getElementById("connect-button");
    const disconnectButton = document.getElementById("disconnect-button");
    const txModalButtons = document.getElementsByClassName("button-tx-modal");

    if (wallet.isConnected) {
      connectButton.style.display = 'none';
      disconnectButton.style.display = '';
      for (var i = 0; i < txModalButtons.length; i++) {
        txModalButtons[i].style.display = '';
      }
    } else {
      connectButton.style.display = '';
      disconnectButton.style.display = 'none';
      for (var i = 0; i < txModalButtons.length; i++) {
        txModalButtons[i].style.display = 'none';
      }
    }
  }

  async function connectWallet() {
    try {
      await wallet.connect({ onlyIfTrusted: true });
      console.log("Connected to wallet");
      updateUI();
    } catch (err) {
      console.error("Failed to connect to the wallet:", err);
    }
  }

  async function disconnectWallet() {
    try {
      await wallet.disconnect();
      console.log("Disconnected from wallet");
      updateUI();
    } catch (err) {
      console.error("Failed to disconnect the wallet:", err);
    }
  }

  // Attach event listeners
  document.getElementById("connect-button").addEventListener("click", connectWallet);
  document.getElementById("disconnect-button").addEventListener("click", disconnectWallet);

  updateUI(); // Call updateUI to set initial button state

  document.body.addEventListener('htmx:afterSwap', function() {
    let modal = document.querySelector("#tx-modal");
    if (modal) {
      console.log("Found the modal");
      const actionButton = document.querySelector(".modal-content button[data-endpoint]");
      if (actionButton) {
        console.log("Found action button");
        actionButton.addEventListener('click', function() {
          const endpoint = this.getAttribute('data-endpoint');
          handleTransaction(endpoint);

        });
      }
    }
  });
});

