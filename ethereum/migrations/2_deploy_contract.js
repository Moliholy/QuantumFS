const QuantumFS = artifacts.require("QuantumFS");

module.exports = async function (deployer) {
    await deployer.deploy(QuantumFS);
};
