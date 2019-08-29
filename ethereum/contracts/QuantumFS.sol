pragma solidity ^0.5.0;

contract QuantumFS
{
    mapping(address => string[]) private fileSystems;

    function currentRevision()
      public
      view
      returns (string memory)
    {
        uint totalRevisions = totalRevisions();
        require(totalRevisions > 0, "Uninitialized file system");

        uint lastRevision = totalRevisions - 1;
        return getRevision(lastRevision);
    }

    function totalRevisions()
      public
      view
      returns (uint)
    {
        return fileSystems[msg.sender].length;
    }

    function getRevision(uint _revision)
      public
      view
      returns (string memory)
    {
        require(_revision < totalRevisions(), "Invalid revision");
        return fileSystems[msg.sender][_revision];
    }

    function addRevision(string calldata _hash)
      external
    {
        fileSystems[msg.sender].push(_hash);
    }

    function evict()
      external
    {
        delete fileSystems[msg.sender];
    }
}
